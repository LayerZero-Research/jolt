use crate::poly::{
    coefficient_layout::CoefficientLayout,
    commitment::{
        commitment_scheme::CommitmentContext,
        layout::{
            CommitmentLayout, LayoutDescriptor, LayoutError, LayoutPublicInputs, LogicalDimensions,
        },
    },
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

    pub fn main_log_embedding(self) -> usize {
        self.embedding.main_log_embedding
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

    pub fn logical_dimensions(self) -> LogicalDimensions {
        LogicalDimensions {
            log_k: self.k().log_2(),
            log_t: self.embedding.stored_t.log_2(),
            main_log_embedding: (self.context == DoryContext::Main)
                .then_some(self.embedding.main_log_embedding),
        }
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

    fn max_setup_vars(&self) -> usize {
        self.shape.num_columns.log_2() + self.shape.num_rows.log_2()
    }
}
