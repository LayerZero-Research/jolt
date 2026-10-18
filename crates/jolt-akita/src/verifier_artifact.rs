//! Offline verifier commitments transported with their exact schedule catalog.

use std::sync::Arc;

use akita_config::{CommitmentConfig, SetupRequirements, TrustedScheduleCatalog};
use akita_params::verifier_setup_matrix_capacity_for_schedule;
use akita_pcs::AkitaDeserialize;
use akita_types::{
    derive_public_matrix_prefix, sample_akita_setup_seed, AkitaExpandedSetup, AkitaSetupDescriptor,
    SetupPrefixVerifierRegistry, MAX_GENERIC_SETUP_DECODE_FIELD_ELEMENTS,
};
use jolt_openings::OpeningsError;
use serde::{Deserialize, Serialize};

use crate::adapters::{invalid_setup, serialize_akita, AkitaBackendVerifierSetup, AkitaField};

/// Callers must authenticate these bytes through the containing bundle's pinned digest.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AkitaVerifierBackendArtifact {
    version: u32,
    pub(crate) schedule: Vec<u8>,
    pub(crate) prefixes: Vec<u8>,
}

impl AkitaVerifierBackendArtifact {
    pub(crate) fn new(schedule: Vec<u8>) -> Self {
        Self {
            version: 1,
            schedule,
            prefixes: Vec::new(),
        }
    }

    /// Called only by offline setup, before a verifier bundle is published.
    pub(crate) fn set_prefixes(
        &mut self,
        verifier: &AkitaBackendVerifierSetup,
    ) -> Result<(), OpeningsError> {
        self.prefixes = serialize_akita(verifier.prefix_slots())?;
        Ok(())
    }

    fn decode_prefixes(
        bytes: &[u8],
    ) -> Result<SetupPrefixVerifierRegistry<AkitaField>, OpeningsError> {
        let registry = SetupPrefixVerifierRegistry::deserialize_compressed_exact(bytes, &())
            .map_err(invalid_setup)?;
        if registry.setup_seed() != &sample_akita_setup_seed() {
            return Err(invalid_setup(
                "offline setup-prefix artifact has the wrong setup seed",
            ));
        }
        Ok(registry)
    }

    pub(crate) fn build<Cfg: CommitmentConfig<Field = AkitaField>>(
        &self,
        catalog: &TrustedScheduleCatalog<Cfg>,
        max_num_vars: usize,
        max_num_batched_polys: usize,
    ) -> Result<AkitaBackendVerifierSetup, OpeningsError> {
        if self.version != 1 {
            return Err(invalid_setup(
                "unsupported offline verifier artifact version",
            ));
        }
        let registry = Self::decode_prefixes(&self.prefixes)?;
        // This scans catalog metadata only; it does not allocate a prover setup.
        let requirements =
            SetupRequirements::from_catalog(catalog, max_num_vars, max_num_batched_polys)
                .map_err(invalid_setup)?;
        for id in requirements.prefix_slot_ids() {
            if registry.get(id).is_none() {
                return Err(invalid_setup(format!(
                    "missing offline setup-prefix commitment: {id:?}"
                )));
            }
        }
        let mut capacity = None;
        for row in catalog
            .rows_within_setup_capacity(max_num_vars, max_num_batched_polys)
            .map_err(invalid_setup)?
        {
            let layout = row.profiles().opening_layout().map_err(invalid_setup)?;
            let required = verifier_setup_matrix_capacity_for_schedule(row.schedule(), &layout)
                .map_err(invalid_setup)?;
            capacity = Some(
                capacity.map_or(required.num_field_elements, |current: usize| {
                    current.max(required.num_field_elements)
                }),
            );
        }
        let num_field_elements = capacity
            .ok_or_else(|| invalid_setup("verifier setup has no admitted schedule rows"))?;
        if num_field_elements > MAX_GENERIC_SETUP_DECODE_FIELD_ELEMENTS {
            return Err(invalid_setup("verifier matrix prefix exceeds setup limit"));
        }
        let setup_seed = sample_akita_setup_seed();
        let matrix = derive_public_matrix_prefix(num_field_elements, &setup_seed);
        let descriptor = AkitaSetupDescriptor {
            max_num_vars,
            max_num_batched_polys,
            num_field_elements,
            setup_seed,
        };
        // The matrix was freshly derived from this descriptor's seed.
        let expanded =
            AkitaExpandedSetup::from_trusted_seed_derived_parts_unchecked(descriptor, matrix);
        AkitaBackendVerifierSetup::from_parts(Arc::new(expanded), registry).map_err(invalid_setup)
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "tests construct valid offline fixtures")]
    #![expect(
        clippy::indexing_slicing,
        reason = "tests truncate a fixture within its bounds"
    )]

    use super::*;

    #[test]
    fn prefix_artifact_rejects_truncation_trailing_bytes_and_wrong_seed() {
        let registry = SetupPrefixVerifierRegistry::<AkitaField>::new(sample_akita_setup_seed());
        let bytes = serialize_akita(&registry).unwrap();
        assert_eq!(
            AkitaVerifierBackendArtifact::decode_prefixes(&bytes).unwrap(),
            registry
        );
        for len in 0..bytes.len() {
            assert!(AkitaVerifierBackendArtifact::decode_prefixes(&bytes[..len]).is_err());
        }
        let mut trailing = bytes;
        trailing.push(0);
        assert!(AkitaVerifierBackendArtifact::decode_prefixes(&trailing).is_err());
        let wrong = serialize_akita(&SetupPrefixVerifierRegistry::<AkitaField>::new(
            [0; 32].into(),
        ))
        .unwrap();
        assert!(AkitaVerifierBackendArtifact::decode_prefixes(&wrong)
            .unwrap_err()
            .to_string()
            .contains("wrong setup seed"));
    }
}
