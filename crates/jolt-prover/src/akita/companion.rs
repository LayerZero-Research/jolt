use jolt_field::JoltField;
use jolt_openings::CommitmentScheme;

/// Optional observer for an Akita PCS implementation that owns the trace
/// outside the Rust witness plane.
///
/// The modular prover remains the protocol driver. A companion may mirror the
/// native stage-0 commitment and stage-8 opening, but it does not own transcript
/// sequencing or any of stages 1--7.
pub trait AkitaPcsCompanion<F, PCS>
where
    F: JoltField,
    PCS: CommitmentScheme<Field = F>,
{
    /// Start the proof-scoped companion bracket and return its public trace
    /// commitment. `None` disables companion comparison for this proof.
    fn commit_trace(
        &mut self,
        setup: &PCS::ProverSetup,
        layout_digest: [u8; 32],
        root_is_grouped: bool,
    ) -> Result<Option<PCS::Output>, String>;

    /// Mirror the main trace opening selected by stage 8.
    fn open_trace(&mut self, point: &[F], evaluation: F) -> Result<(), String>;

    /// Close any bracket opened by [`Self::commit_trace`]. Must be idempotent.
    fn shutdown(&mut self) -> Result<(), String>;
}

/// The ordinary Rust-only path pays no companion cost.
pub struct NoAkitaPcsCompanion;

impl<F, PCS> AkitaPcsCompanion<F, PCS> for NoAkitaPcsCompanion
where
    F: JoltField,
    PCS: CommitmentScheme<Field = F>,
{
    fn commit_trace(
        &mut self,
        _setup: &PCS::ProverSetup,
        _layout_digest: [u8; 32],
        _root_is_grouped: bool,
    ) -> Result<Option<PCS::Output>, String> {
        Ok(None)
    }

    fn open_trace(&mut self, _point: &[F], _evaluation: F) -> Result<(), String> {
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), String> {
        Ok(())
    }
}
