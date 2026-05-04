use know_now_contract::contract::GeneratorContract;

use crate::artifact::ArtifactDescriptor;

/// Errors returned by generators during artifact production.
#[derive(Debug, Clone)]
pub struct GenerationError {
    pub code: String,
    pub message: String,
}

impl std::fmt::Display for GenerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

/// Trait that all built-in generators implement.
pub trait Generator: Send + Sync {
    /// Human-readable generator name (e.g., "know_now_gen_postgres").
    fn name(&self) -> &str;

    /// Generator version (semver).
    fn version(&self) -> &str;

    /// Produce artifact descriptors from the validated contract.
    ///
    /// # Errors
    ///
    /// Returns `GenerationError` if generation fails.
    fn generate(
        &self,
        contract: &GeneratorContract,
    ) -> Result<Vec<ArtifactDescriptor>, Vec<GenerationError>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubGenerator;

    impl Generator for StubGenerator {
        fn name(&self) -> &str {
            "stub"
        }

        fn version(&self) -> &str {
            "0.0.0"
        }

        fn generate(
            &self,
            _contract: &GeneratorContract,
        ) -> Result<Vec<ArtifactDescriptor>, Vec<GenerationError>> {
            Ok(vec![])
        }
    }

    #[test]
    fn stub_generator_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<StubGenerator>();
    }

    #[test]
    fn stub_generator_produces_empty() {
        let gen = StubGenerator;
        let contract = GeneratorContract {
            contract_version: "1.0".into(),
            project: None,
            target_database: None,
            entities: vec![],
            relationships: vec![],
            source_systems: vec![],
            quality_rules: vec![],
            governance: None,
            open_questions: vec![],
            assumptions: vec![],
            trace: Default::default(),
        };
        let result = gen.generate(&contract).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn generation_error_display() {
        let err = GenerationError {
            code: "GEN-001".into(),
            message: "test error".into(),
        };
        assert_eq!(err.to_string(), "GEN-001: test error");
    }
}
