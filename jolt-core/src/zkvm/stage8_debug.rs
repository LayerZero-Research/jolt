use crate::field::JoltField;
use crate::poly::commitment::dory::DoryGlobals;
use crate::poly::opening_proof::{OpeningPoint, SumcheckId, BIG_ENDIAN};
use crate::utils::math::Math;
use crate::zkvm::witness::CommittedPolynomial;

/// Comma-separated env var controlling which polynomial classes are included in Stage-8 Dory batch.
///
/// Examples:
/// - `JOLT_DEBUG_DORY_CLASSES=bytecode`
/// - `JOLT_DEBUG_DORY_CLASSES=bytecode,program_image`
/// - `JOLT_DEBUG_DORY_CLASSES=ra,advice,dense`
/// - `JOLT_DEBUG_DORY_CLASSES=all`
pub(crate) const STAGE8_DORY_CLASSES_ENV: &str = "JOLT_DEBUG_DORY_CLASSES";

#[derive(Clone, Copy, Debug)]
pub(crate) struct Stage8DoryClassFilter {
    pub dense: bool,
    pub ra: bool,
    pub advice: bool,
    pub bytecode: bool,
    pub program_image: bool,
}

impl Stage8DoryClassFilter {
    #[inline]
    pub(crate) fn all_enabled() -> Self {
        Self {
            dense: true,
            ra: true,
            advice: true,
            bytecode: true,
            program_image: true,
        }
    }

    pub(crate) fn from_env() -> Self {
        let Ok(raw) = std::env::var(STAGE8_DORY_CLASSES_ENV) else {
            return Self::all_enabled();
        };
        let raw = raw.trim();
        if raw.is_empty() {
            return Self::all_enabled();
        }

        let mut filter = Self {
            dense: false,
            ra: false,
            advice: false,
            bytecode: false,
            program_image: false,
        };

        for token in raw.split(',').map(|s| s.trim().to_ascii_lowercase()) {
            match token.as_str() {
                "all" => return Self::all_enabled(),
                "dense" => filter.dense = true,
                "ra" => filter.ra = true,
                "advice" => filter.advice = true,
                "bytecode" => filter.bytecode = true,
                "program_image" => filter.program_image = true,
                "" => {}
                other => {
                    panic!(
                        "Unknown Stage-8 Dory class '{other}' in {STAGE8_DORY_CLASSES_ENV}. \
Supported: all,dense,ra,advice,bytecode,program_image"
                    );
                }
            }
        }

        filter
    }

    #[inline]
    pub(crate) fn any_enabled(&self) -> bool {
        self.dense || self.ra || self.advice || self.bytecode || self.program_image
    }

    #[inline]
    pub(crate) fn includes_poly(&self, poly: CommittedPolynomial) -> bool {
        match poly {
            CommittedPolynomial::RamInc | CommittedPolynomial::RdInc => self.dense,
            CommittedPolynomial::InstructionRa(_)
            | CommittedPolynomial::BytecodeRa(_)
            | CommittedPolynomial::RamRa(_) => self.ra,
            CommittedPolynomial::TrustedAdvice | CommittedPolynomial::UntrustedAdvice => self.advice,
            CommittedPolynomial::BytecodeChunk(_) => self.bytecode,
            CommittedPolynomial::ProgramImageInit => self.program_image,
        }
    }
}

#[inline]
pub(crate) fn report_stage8_direct_claim_check<F: JoltField + core::fmt::Debug>(
    poly: CommittedPolynomial,
    source_sumcheck: SumcheckId,
    direct_eval: F,
    source_claim: F,
    lagrange_factor: F,
    staged_claim: F,
) {
    let expected_staged = direct_eval * lagrange_factor;

    debug_assert_eq!(
        direct_eval, source_claim,
        "Stage8 direct/source mismatch for {:?} from {:?}",
        poly, source_sumcheck
    );
    debug_assert_eq!(
        expected_staged, staged_claim,
        "Stage8 staged-claim mismatch for {:?} from {:?}",
        poly, source_sumcheck
    );
}

pub(crate) fn derive_poly_source_point_from_matrix_dims<F: JoltField>(
    stage8_opening_point: &OpeningPoint<BIG_ENDIAN, F>,
    poly_num_rows: usize,
    poly_num_columns: usize,
) -> OpeningPoint<BIG_ENDIAN, F> {
    assert!(
        poly_num_rows.is_power_of_two() && poly_num_columns.is_power_of_two(),
        "polynomial matrix dimensions must be powers of two (rows={poly_num_rows}, cols={poly_num_columns})"
    );
    let nu_poly = poly_num_rows.log_2();
    let sigma_poly = poly_num_columns.log_2();
    let nu_full = DoryGlobals::get_max_num_rows().log_2();
    let sigma_full = DoryGlobals::get_num_columns().log_2();
    assert!(
        sigma_poly <= sigma_full && nu_poly <= nu_full,
        "top-left projection requires poly dims <= full dims (poly sigma/nu={sigma_poly}/{nu_poly}, full sigma/nu={sigma_full}/{nu_full})"
    );

    // Dimension-only projection:
    // - Treat full point as [row_variables || column_variables]
    // - For target dims (nu_poly rows, sigma_poly cols), take tails:
    //   [last nu_poly row vars || last sigma_poly col vars]
    let row_be = &stage8_opening_point.r[..nu_full];
    let col_be = &stage8_opening_point.r[nu_full..nu_full + sigma_full];
    let row_tail = &row_be[nu_full - nu_poly..];
    let col_tail = &col_be[sigma_full - sigma_poly..];

    let mut projected = Vec::with_capacity(nu_poly + sigma_poly);
    projected.extend_from_slice(row_tail);
    projected.extend_from_slice(col_tail);
    OpeningPoint::<BIG_ENDIAN, F>::new(projected)
}

#[inline]
pub(crate) fn derive_poly_source_point_from_dory_dims<F: JoltField>(
    stage8_opening_point: &OpeningPoint<BIG_ENDIAN, F>,
    poly_num_vars: usize,
) -> OpeningPoint<BIG_ENDIAN, F> {
    let (sigma_poly, nu_poly) = DoryGlobals::balanced_sigma_nu(poly_num_vars);
    let poly_num_rows = 1usize << nu_poly;
    let poly_num_columns = 1usize << sigma_poly;
    derive_poly_source_point_from_matrix_dims(
        stage8_opening_point,
        poly_num_rows,
        poly_num_columns,
    )
}


