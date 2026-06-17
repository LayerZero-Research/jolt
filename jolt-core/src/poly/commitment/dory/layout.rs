use crate::poly::{
    coefficient_layout::CoefficientLayout,
    commitment::{
        commitment_scheme::CommitmentContext,
        layout::{CommitmentLayout, LayoutDescriptor, LayoutError, LayoutPublicInputs},
    },
    multilinear_polynomial::MultilinearPolynomial,
};
use crate::utils::math::Math;

use super::dory_globals::{DoryContext, DoryLayout};

const DORY_LAYOUT_SCHEME_TAG: u16 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DoryMatrixShape {
    num_columns: usize,
    num_rows: usize,
}

impl DoryMatrixShape {
    fn balanced(total_vars: usize) -> Self {
        let sigma = total_vars.div_ceil(2);
        let nu = total_vars - sigma;
        Self {
            num_columns: 1usize << sigma,
            num_rows: 1usize << nu,
        }
    }

    fn sigma_nu(self) -> (usize, usize) {
        (self.num_columns.log_2(), self.num_rows.log_2())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DoryEmbedding {
    stored_t: usize,
    embedded_t: usize,
    main_log_embedding: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoryCommitmentLayout {
    context: DoryContext,
    orientation: DoryLayout,
    k: usize,
    shape: DoryMatrixShape,
    embedding: DoryEmbedding,
}

impl DoryCommitmentLayout {
    pub fn main(k: usize, trace_len: usize, main_log_embedding: usize, layout: DoryLayout) -> Self {
        let log_k = k.log_2();
        let embedded_t = 1usize << main_log_embedding.saturating_sub(log_k);
        let shape = DoryMatrixShape::balanced(log_k + embedded_t.log_2());
        Self {
            context: DoryContext::Main,
            orientation: layout,
            k,
            shape,
            embedding: DoryEmbedding {
                stored_t: trace_len,
                embedded_t,
                main_log_embedding,
            },
        }
    }

    pub fn advice(context: DoryContext, len: usize) -> Self {
        debug_assert!(matches!(
            context,
            DoryContext::TrustedAdvice | DoryContext::UntrustedAdvice
        ));
        let len = len.next_power_of_two().max(1);
        let shape = DoryMatrixShape::balanced(len.log_2());
        Self {
            context,
            orientation: DoryLayout::CycleMajor,
            k: 1,
            shape,
            embedding: DoryEmbedding {
                stored_t: len,
                embedded_t: len,
                main_log_embedding: len.log_2(),
            },
        }
    }

    /// Balanced layout for committing a single standalone polynomial (not the main RLC matrix).
    /// A one-hot poly occupies its full `K x T`; any other poly is treated as a flat `k = 1` vector.
    pub fn for_polynomial(
        poly: &MultilinearPolynomial<ark_bn254::Fr>,
        orientation: DoryLayout,
    ) -> Self {
        match poly {
            MultilinearPolynomial::OneHot(poly) => Self::main(
                poly.K,
                poly.nonzero_indices.len(),
                poly.get_num_vars(),
                orientation,
            ),
            _ => {
                let len = poly.original_len().next_power_of_two().max(1);
                Self::main(1, len, len.log_2(), orientation)
            }
        }
    }

    pub fn from_context(config: &DoryLayout, context: CommitmentContext) -> Self {
        match context {
            CommitmentContext::MainTrace {
                k,
                trace_len,
                commitment_total_vars,
            } => Self::main(k, trace_len, commitment_total_vars, *config),
            CommitmentContext::TrustedAdvice { len } => {
                Self::advice(DoryContext::TrustedAdvice, len)
            }
            CommitmentContext::UntrustedAdvice { len } => {
                Self::advice(DoryContext::UntrustedAdvice, len)
            }
        }
    }

    pub fn context(self) -> DoryContext {
        self.context
    }

    pub fn orientation(self) -> DoryLayout {
        self.orientation
    }

    pub fn matrix_shape(self) -> (usize, usize) {
        (self.shape.num_rows, self.shape.num_columns)
    }

    pub fn sigma_nu(self) -> (usize, usize) {
        self.shape.sigma_nu()
    }

    pub fn num_columns(self) -> usize {
        self.shape.num_columns
    }

    pub fn num_rows(self) -> usize {
        self.shape.num_rows
    }

    pub fn stored_t(self) -> usize {
        self.embedding.stored_t
    }

    pub fn embedded_t(self) -> usize {
        self.embedding.embedded_t
    }

    pub fn k(self) -> usize {
        if self.context == DoryContext::Main {
            self.k
        } else {
            let total = self.shape.num_rows * self.shape.num_columns;
            debug_assert_eq!(total % self.embedding.stored_t, 0);
            total / self.embedding.stored_t
        }
    }

    pub fn coefficient_layout(self) -> CoefficientLayout {
        CoefficientLayout {
            num_columns: self.shape.num_columns,
            num_rows: self.shape.num_rows,
            T: self.embedding.stored_t,
            cycle_major: self.orientation == DoryLayout::CycleMajor,
        }
    }

    pub fn one_hot_stride(self) -> usize {
        if self.context != DoryContext::Main || self.orientation != DoryLayout::AddressMajor {
            return 1;
        }
        1usize << self.main_embedding_extra_vars()
    }

    pub fn dense_stride(self) -> usize {
        if self.context != DoryContext::Main || self.orientation != DoryLayout::AddressMajor {
            return 1;
        }
        let dense_stride_log = self.main_embedding_extra_vars() + self.k.log_2();
        1usize << dense_stride_log
    }

    pub fn address_major_cycles_per_row(self) -> usize {
        let k = self.k();
        debug_assert!(k > 0);
        debug_assert_eq!(self.shape.num_columns % k, 0);
        self.shape.num_columns / k
    }

    fn main_embedding_extra_vars(self) -> usize {
        let main_total_vars = self.k.log_2() + self.embedding.stored_t.log_2();
        self.embedding
            .main_log_embedding
            .saturating_sub(main_total_vars)
    }
}

impl CommitmentLayout for DoryCommitmentLayout {
    fn descriptor(&self) -> LayoutDescriptor {
        LayoutDescriptor {
            scheme_tag: DORY_LAYOUT_SCHEME_TAG,
            payload: vec![u8::from(self.orientation)],
        }
    }

    fn validate_descriptor(
        descriptor: &LayoutDescriptor,
        public: &LayoutPublicInputs,
    ) -> Result<Self, LayoutError> {
        if descriptor.scheme_tag != DORY_LAYOUT_SCHEME_TAG {
            return Err(LayoutError::SchemeTagMismatch {
                expected: DORY_LAYOUT_SCHEME_TAG,
                actual: descriptor.scheme_tag,
            });
        }
        let [layout] = descriptor.payload.as_slice() else {
            return Err(LayoutError::MalformedDescriptor);
        };
        if *layout > 1 {
            return Err(LayoutError::MalformedDescriptor);
        }
        let main_log_embedding = public
            .main_log_embedding
            .unwrap_or(public.log_k + public.log_t);
        Ok(Self::main(
            1usize << public.log_k,
            1usize << public.log_t,
            main_log_embedding,
            DoryLayout::from(*layout),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{DoryCommitmentLayout, DoryLayout};
    use crate::poly::commitment::layout::{
        CommitmentLayout, LayoutDescriptor, LayoutError, LayoutPublicInputs,
    };
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};

    fn public() -> LayoutPublicInputs {
        LayoutPublicInputs {
            log_k: 4,
            log_t: 6,
            main_log_embedding: Some(10),
        }
    }

    fn round_trip(descriptor: &LayoutDescriptor, compress: Compress) -> LayoutDescriptor {
        let mut bytes = Vec::new();
        descriptor
            .serialize_with_mode(&mut bytes, compress)
            .unwrap();
        LayoutDescriptor::deserialize_with_mode(&bytes[..], compress, Validate::Yes).unwrap()
    }

    #[test]
    fn descriptor_round_trips_in_both_compress_modes() {
        for orientation in [DoryLayout::CycleMajor, DoryLayout::AddressMajor] {
            let descriptor = DoryCommitmentLayout::main(16, 64, 10, orientation).descriptor();
            for compress in [Compress::Yes, Compress::No] {
                assert_eq!(round_trip(&descriptor, compress), descriptor);
            }
        }
    }

    #[test]
    fn validate_descriptor_recovers_orientation() {
        for orientation in [DoryLayout::CycleMajor, DoryLayout::AddressMajor] {
            let descriptor = DoryCommitmentLayout::main(16, 64, 10, orientation).descriptor();
            let reconstructed =
                DoryCommitmentLayout::validate_descriptor(&descriptor, &public()).unwrap();
            assert_eq!(reconstructed.orientation(), orientation);
        }
    }

    #[test]
    fn validate_descriptor_rejects_wrong_scheme_tag() {
        let descriptor = LayoutDescriptor {
            scheme_tag: 999,
            payload: vec![0],
        };
        assert!(matches!(
            DoryCommitmentLayout::validate_descriptor(&descriptor, &public()),
            Err(LayoutError::SchemeTagMismatch { .. })
        ));
    }

    #[test]
    fn validate_descriptor_rejects_malformed_payload() {
        for payload in [vec![], vec![0u8, 0u8], vec![2u8]] {
            let descriptor = LayoutDescriptor {
                scheme_tag: 1,
                payload,
            };
            assert!(matches!(
                DoryCommitmentLayout::validate_descriptor(&descriptor, &public()),
                Err(LayoutError::MalformedDescriptor)
            ));
        }
    }

    #[test]
    fn tampered_descriptor_flips_reconstructed_orientation() {
        let cycle = DoryCommitmentLayout::main(16, 64, 10, DoryLayout::CycleMajor).descriptor();
        let address = DoryCommitmentLayout::main(16, 64, 10, DoryLayout::AddressMajor).descriptor();
        // The free choice is genuinely carried in the descriptor, so the two differ and each
        // reconstructs to its own orientation. A verifier that binds the descriptor in
        // Fiat-Shamir therefore cannot have the orientation silently swapped.
        assert_ne!(cycle, address);
        assert_eq!(
            DoryCommitmentLayout::validate_descriptor(&cycle, &public())
                .unwrap()
                .orientation(),
            DoryLayout::CycleMajor
        );
        assert_eq!(
            DoryCommitmentLayout::validate_descriptor(&address, &public())
                .unwrap()
                .orientation(),
            DoryLayout::AddressMajor
        );
    }
}
