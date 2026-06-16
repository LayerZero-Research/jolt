use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

#[derive(Clone, Debug, PartialEq, Eq, CanonicalSerialize, CanonicalDeserialize)]
pub struct LayoutDescriptor {
    pub scheme_tag: u16,
    pub payload: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutPublicInputs {
    pub log_k: usize,
    pub log_t: usize,
    pub main_log_embedding: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LayoutError {
    SchemeTagMismatch { expected: u16, actual: u16 },
    MalformedDescriptor,
}

pub trait CommitmentLayout: Clone + Send + Sync + 'static {
    fn descriptor(&self) -> LayoutDescriptor;

    fn validate_descriptor(
        descriptor: &LayoutDescriptor,
        public: &LayoutPublicInputs,
    ) -> Result<Self, LayoutError>
    where
        Self: Sized;

    fn max_setup_vars(&self) -> usize;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NoCommitmentLayout;

impl CommitmentLayout for NoCommitmentLayout {
    fn descriptor(&self) -> LayoutDescriptor {
        LayoutDescriptor {
            scheme_tag: 0,
            payload: Vec::new(),
        }
    }

    fn validate_descriptor(
        descriptor: &LayoutDescriptor,
        _public: &LayoutPublicInputs,
    ) -> Result<Self, LayoutError> {
        if descriptor.scheme_tag != 0 || !descriptor.payload.is_empty() {
            return Err(LayoutError::MalformedDescriptor);
        }
        Ok(Self)
    }

    fn max_setup_vars(&self) -> usize {
        0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolynomialFamily {
    MainTraceOneHot,
    MainTraceDense,
    TrustedAdvice,
    UntrustedAdvice,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogicalDimensions {
    pub log_k: usize,
    pub log_t: usize,
    pub main_log_embedding: Option<usize>,
}
