use std::{fs, path::Path};

const CI_TEMPLATES: &[&str] = &[
    "examples/ci/github-actions/know-now.yml",
    "examples/ci/gitlab-ci/.gitlab-ci.yml",
    "examples/ci/circleci/config.yml",
    "examples/ci/buildkite/pipeline.yml",
];

#[test]
fn ci_templates_parse_as_valid_yaml() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");

    for template in CI_TEMPLATES {
        let path = repo_root.join(template);
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {template}: {err}"));

        let result: Result<serde_json::Value, _> = serde_saphyr::from_str(&content);
        result.unwrap_or_else(|err| panic!("{template} is not valid YAML: {err}"));
    }
}
