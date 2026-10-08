//! A fuzzer-programmable guest: a small interpreter whose input is a program.
//!
//! The harness cannot synthesize Rust guests per iteration, so this one guest
//! turns input bytes into RV64IMAC behavior Jolt must prove: register-level
//! arithmetic with the exact RISC-V instructions (division by zero, signed
//! overflow, high multiplies, 32-bit `W` forms, shifts by out-of-range amounts),
//! loads and stores of every width over a heap span the program chooses (which
//! sets the RAM domain), atomics, advice reads, bounded loops (which set the
//! trace length), and a deliberate panic. Every execution of every input is a
//! statement Jolt must prove, including the panicking ones.
//!
//! The provable functions differ only in memory attributes, so one program can
//! be proved under several layouts (advice capacities, heap sizes).
#![cfg_attr(feature = "guest", no_std)]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use core::hint::black_box;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use jolt::{TrustedAdvice, UntrustedAdvice};

/// One register-register RISC-V instruction on 64-bit operands.
#[cfg(target_arch = "riscv64")]
macro_rules! rr {
    ($insn:literal, $lhs:expr, $rhs:expr) => {{
        let result: u64;
        // SAFETY: register-only arithmetic, no memory or control-flow effects.
        unsafe {
            core::arch::asm!(
                concat!($insn, " {result}, {lhs}, {rhs}"),
                result = lateout(reg) result,
                lhs = in(reg) $lhs,
                rhs = in(reg) $rhs,
                options(nomem, nostack, pure)
            );
        }
        result
    }};
}

/// Host builds never execute the interpreter; keep them compiling.
#[cfg(not(target_arch = "riscv64"))]
macro_rules! rr {
    ($insn:literal, $lhs:expr, $rhs:expr) => {{
        let _ = ($lhs, $rhs);
        0u64
    }};
}

fn alu(op: u8, a: u64, b: u64) -> u64 {
    match op % 32 {
        0 => rr!("add", a, b),
        1 => rr!("sub", a, b),
        2 => rr!("mul", a, b),
        3 => rr!("mulh", a, b),
        4 => rr!("mulhu", a, b),
        5 => rr!("mulhsu", a, b),
        6 => rr!("div", a, b),
        7 => rr!("divu", a, b),
        8 => rr!("rem", a, b),
        9 => rr!("remu", a, b),
        10 => rr!("addw", a, b),
        11 => rr!("subw", a, b),
        12 => rr!("mulw", a, b),
        13 => rr!("divw", a, b),
        14 => rr!("divuw", a, b),
        15 => rr!("remw", a, b),
        16 => rr!("remuw", a, b),
        17 => rr!("sll", a, b),
        18 => rr!("srl", a, b),
        19 => rr!("sra", a, b),
        20 => rr!("sllw", a, b),
        21 => rr!("srlw", a, b),
        22 => rr!("sraw", a, b),
        23 => rr!("slt", a, b),
        24 => rr!("sltu", a, b),
        25 => rr!("and", a, b),
        26 => rr!("or", a, b),
        27 => rr!("xor", a, b),
        28 => a.rotate_left((b & 63) as u32),
        29 => a.leading_zeros() as u64 ^ b.count_ones() as u64,
        30 => a.wrapping_neg(),
        _ => (a as i64).wrapping_abs() as u64,
    }
}

/// Operand edge values the fuzzer reaches by one byte.
fn constant(selector: u8, payload: u64) -> u64 {
    match selector % 12 {
        0 => 0,
        1 => 1,
        2 => u64::MAX,
        3 => i64::MIN as u64,
        4 => i64::MAX as u64,
        5 => i32::MIN as i64 as u64,
        6 => u32::MAX as u64,
        7 => 1 << (payload % 64),
        8 => (1 << (payload % 64)) - 1,
        9 => payload as u32 as i32 as i64 as u64,
        10 => payload & 0xffff,
        _ => payload,
    }
}

struct Machine<'a> {
    code: &'a [u8],
    pc: usize,
    regs: [u64; 8],
    heap: Vec<u64>,
    trusted: &'a [u8],
    untrusted: &'a [u8],
    budget: u32,
    digest: u64,
    depth: u8,
}

impl Machine<'_> {
    fn byte(&mut self) -> u8 {
        let value = self.code.get(self.pc).copied().unwrap_or(0);
        self.pc += 1;
        value
    }

    fn word(&mut self) -> u64 {
        let mut bytes = [0u8; 8];
        for byte in &mut bytes {
            *byte = self.byte();
        }
        u64::from_le_bytes(bytes)
    }

    fn reg(&mut self) -> usize {
        usize::from(self.byte() % 8)
    }

    /// Byte offset of a `width`-byte access inside the heap, width-aligned.
    fn offset(&self, address: u64, width: usize) -> usize {
        let bytes = self.heap.len() * 8;
        (address as usize % (bytes - width + 1)) & !(width - 1)
    }

    fn advice(source: &[u8], index: u64) -> u64 {
        if source.is_empty() {
            return 0;
        }
        let start = index as usize % source.len();
        let mut bytes = [0u8; 8];
        for (offset, byte) in bytes.iter_mut().enumerate() {
            *byte = source.get(start + offset).copied().unwrap_or(0);
        }
        u64::from_le_bytes(bytes)
    }

    fn step(&mut self) {
        let op = self.byte();
        match op % 10 {
            0 => {
                let (dst, selector) = (self.reg(), self.byte());
                let payload = self.word();
                self.regs[dst] = black_box(constant(selector, payload));
            }
            1..=3 => {
                let (dst, lhs, rhs, alu_op) = (self.reg(), self.reg(), self.reg(), self.byte());
                self.regs[dst] = alu(alu_op, black_box(self.regs[lhs]), black_box(self.regs[rhs]));
            }
            4 => self.store(),
            5 => self.load(),
            6 => {
                // Repeat the next `body` bytes `count` times.
                let (count, body) = (self.byte(), usize::from(self.byte() % 32) + 1);
                let start = self.pc;
                // Bounded nesting keeps the guest stack small.
                let count = if self.depth < 8 { count } else { 0 };
                self.depth += 1;
                for _ in 0..count {
                    self.pc = start;
                    while self.pc < start + body && self.budget > 0 {
                        self.budget -= 1;
                        self.step();
                    }
                }
                self.depth -= 1;
                self.pc = start + body;
            }
            7 => {
                let (dst, index, trusted) = (self.reg(), self.reg(), self.byte() % 2 == 0);
                let source = if trusted {
                    self.trusted
                } else {
                    self.untrusted
                };
                self.regs[dst] = Self::advice(source, self.regs[index]);
            }
            8 => self.atomic(),
            _ => {
                let (lhs, rhs, kind) = (self.reg(), self.reg(), self.byte());
                if kind % 16 == 0 && self.regs[lhs] == self.regs[rhs] {
                    panic!("interp: requested panic");
                }
                self.digest = self.digest.rotate_left(7) ^ self.regs[lhs];
            }
        }
    }

    fn store(&mut self) {
        let (src, address, width) = (self.reg(), self.reg(), 1usize << (self.byte() % 4));
        let value = self.regs[src];
        let offset = self.offset(self.regs[address], width);
        let base = self.heap.as_mut_ptr().cast::<u8>();
        // SAFETY: `offset + width` is inside the heap buffer and width-aligned,
        // so every access below is in bounds and aligned for its type.
        unsafe {
            let at = base.add(offset);
            match width {
                1 => core::ptr::write_volatile(at, value as u8),
                2 => core::ptr::write_volatile(at.cast::<u16>(), value as u16),
                4 => core::ptr::write_volatile(at.cast::<u32>(), value as u32),
                _ => core::ptr::write_volatile(at.cast::<u64>(), value),
            }
        }
    }

    fn load(&mut self) {
        let (dst, address, kind) = (self.reg(), self.reg(), self.byte());
        let width = 1usize << (kind % 4);
        let signed = kind & 4 != 0;
        let offset = self.offset(self.regs[address], width);
        let base = self.heap.as_ptr().cast::<u8>();
        // SAFETY: as in `store`.
        self.regs[dst] = unsafe {
            let at = base.add(offset);
            match (width, signed) {
                (1, false) => u64::from(core::ptr::read_volatile(at)),
                (1, true) => core::ptr::read_volatile(at.cast::<i8>()) as u64,
                (2, false) => u64::from(core::ptr::read_volatile(at.cast::<u16>())),
                (2, true) => core::ptr::read_volatile(at.cast::<i16>()) as u64,
                (4, false) => u64::from(core::ptr::read_volatile(at.cast::<u32>())),
                (4, true) => core::ptr::read_volatile(at.cast::<i32>()) as u64,
                _ => core::ptr::read_volatile(at.cast::<u64>()),
            }
        };
    }

    fn atomic(&mut self) {
        let (dst, address, src, kind) = (self.reg(), self.reg(), self.reg(), self.byte());
        let wide = kind & 8 == 0;
        let width = if wide { 8 } else { 4 };
        let offset = self.offset(self.regs[address], width);
        let value = self.regs[src];
        let base = self.heap.as_mut_ptr().cast::<u8>();
        // SAFETY: in bounds and aligned (see `offset`); the heap outlives the call.
        self.regs[dst] = unsafe {
            if wide {
                let cell = &*base.add(offset).cast::<AtomicU64>();
                match kind % 8 {
                    0 => cell.fetch_add(value, Ordering::SeqCst),
                    1 => cell.swap(value, Ordering::SeqCst),
                    2 => cell.fetch_and(value, Ordering::SeqCst),
                    3 => cell.fetch_or(value, Ordering::SeqCst),
                    4 => cell.fetch_xor(value, Ordering::SeqCst),
                    5 => cell.fetch_max(value, Ordering::SeqCst),
                    6 => cell.fetch_min(value, Ordering::SeqCst),
                    _ => match cell.compare_exchange(
                        value,
                        !value,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    ) {
                        Ok(previous) | Err(previous) => previous,
                    },
                }
            } else {
                let cell = &*base.add(offset).cast::<AtomicU32>();
                let value = value as u32;
                u64::from(match kind % 8 {
                    0 => cell.fetch_add(value, Ordering::SeqCst),
                    1 => cell.swap(value, Ordering::SeqCst),
                    2 => cell.fetch_and(value, Ordering::SeqCst),
                    3 => cell.fetch_or(value, Ordering::SeqCst),
                    4 => cell.fetch_xor(value, Ordering::SeqCst),
                    5 => cell.fetch_max(value, Ordering::SeqCst),
                    6 => cell.fetch_min(value, Ordering::SeqCst),
                    _ => match cell.compare_exchange(
                        value,
                        !value,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    ) {
                        Ok(previous) | Err(previous) => previous,
                    },
                })
            }
        };
    }
}

/// Program layout: `[heap_words: u32][budget: u32][code...]`. The heap is at
/// least one word and at most `max_heap_words`; the step budget bounds the
/// work (and so the trace) whatever loops the code contains.
pub fn run(program: &[u8], trusted: &[u8], untrusted: &[u8], max_heap_words: usize) -> u64 {
    let header = |at: usize| {
        let mut bytes = [0u8; 4];
        for (offset, byte) in bytes.iter_mut().enumerate() {
            *byte = program.get(at + offset).copied().unwrap_or(0);
        }
        u32::from_le_bytes(bytes)
    };
    let heap_words = (header(0) as usize % max_heap_words).max(1);
    let budget = header(4);
    let mut machine = Machine {
        code: program.get(8..).unwrap_or(&[]),
        pc: 0,
        regs: [0; 8],
        heap: vec![0u64; heap_words],
        trusted,
        untrusted,
        budget,
        digest: 0,
        depth: 0,
    };
    while machine.pc < machine.code.len() && machine.budget > 0 {
        machine.budget -= 1;
        machine.step();
    }
    machine
        .regs
        .iter()
        .fold(machine.digest, |acc, reg| acc.rotate_left(9) ^ reg)
}

/// Default layout: 4 KiB input and advice, 1 MiB heap.
#[jolt::provable(heap_size = 1048576, stack_size = 65536, max_input_size = 4096)]
fn interp(
    program: Vec<u8>,
    trusted: TrustedAdvice<Vec<u8>>,
    untrusted: UntrustedAdvice<Vec<u8>>,
) -> u64 {
    run(&program, &trusted, &untrusted, 1 << 16)
}

/// No advice arguments (the program takes no precommitted advice objects).
#[jolt::provable(heap_size = 1048576, stack_size = 65536, max_input_size = 4096)]
fn interp_plain(program: Vec<u8>) -> u64 {
    run(&program, &[], &[], 1 << 16)
}

/// Large advice capacities: physical arities 17 (1 MiB) and 23 (64 MiB),
/// the latter above the smallest trace's final arity (22 at K=16).
#[jolt::provable(
    heap_size = 1048576,
    stack_size = 65536,
    max_input_size = 4096,
    max_trusted_advice_size = 67108864,
    max_untrusted_advice_size = 1048576
)]
fn interp_advice_large(
    program: Vec<u8>,
    trusted: TrustedAdvice<Vec<u8>>,
    untrusted: UntrustedAdvice<Vec<u8>>,
) -> u64 {
    run(&program, &trusted, &untrusted, 1 << 16)
}

/// A 256 MiB heap: programs can touch a RAM domain of `2^25` words.
#[jolt::provable(heap_size = 268435456, stack_size = 65536, max_input_size = 4096)]
fn interp_big_heap(program: Vec<u8>) -> u64 {
    run(&program, &[], &[], 1 << 25)
}
