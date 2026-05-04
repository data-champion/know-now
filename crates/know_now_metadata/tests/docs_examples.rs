use std::{fs, path::Path};

use know_now_metadata::{
    authoring::AuthoringMetadata, budgets::ParserBudgets, parser::parse_yaml, span::SourceId,
};

const DOC_FILES: &[&str] = &[
    "metadata-reference.md",
    "yaml-subset.md",
    "logical-types.md",
    "semantic-types.md",
    "governance.md",
    "open-questions-assumptions.md",
    "domains-modules.md",
];

const REQUIRED_METADATA_REFERENCE_TOKENS: &[&str] = &[
    "`version`",
    "`project`",
    "`target_database`",
    "`policy`",
    "`domains`",
    "`modules`",
    "`entities`",
    "`relationships`",
    "`sources`",
    "`rules`",
    "`governance`",
    "`open_questions`",
    "`assumptions`",
    "`name`",
    "`description`",
    "`owner`",
    "`tags`",
    "`kind`",
    "`compatibility_floor`",
    "`pack`",
    "`display_name`",
    "`domain`",
    "`module`",
    "`steward`",
    "`classification`",
    "`retention_policy`",
    "`type`",
    "`business_key`",
    "`attributes`",
    "`logical_type`",
    "`semantic_type`",
    "`sensitivity`",
    "`pii`",
    "`required`",
    "`unique`",
    "`constraints`",
    "`from_entity`",
    "`to_entity`",
    "`cardinality`",
    "`from_key`",
    "`to_key`",
    "`tables`",
    "`entity`",
    "`schema`",
    "`columns`",
    "`source`",
    "`target`",
    "`transform`",
    "`attribute`",
    "`rule_type`",
    "`expression`",
    "`severity`",
    "`data_owner`",
    "`data_steward`",
    "`classification_default`",
    "`retention_default`",
    "`question`",
    "`context`",
    "`priority`",
    "`statement`",
    "`rationale`",
    "`risk`",
];

#[derive(Debug)]
struct YamlBlock {
    content: String,
    expected_error: Option<String>,
    line: usize,
}

#[test]
fn markdown_yaml_examples_validate_with_parser_rules() {
    let user_docs_dir = user_docs_dir();
    let budgets = ParserBudgets::default();
    let mut source_counter: u32 = 0;

    for doc in DOC_FILES {
        let path = user_docs_dir.join(doc);
        let markdown = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let blocks = extract_yaml_blocks(&markdown);

        assert!(
            !blocks.is_empty(),
            "expected YAML blocks in {}",
            path.display()
        );

        for block in blocks {
            source_counter = source_counter.saturating_add(1);
            let parse_result = parse_yaml::<AuthoringMetadata>(
                &block.content,
                Path::new(doc),
                SourceId(source_counter),
                &budgets,
            );

            if let Some(expected_code) = block.expected_error {
                let errors = parse_result.unwrap_err_or_else(|_| {
                    panic!(
                        "expected parse failure with code {expected_code} in {doc}:{}",
                        block.line
                    )
                });
                assert!(
                    errors.iter().any(|err| err.kind.code() == expected_code),
                    "expected error code {expected_code} in {doc}:{}",
                    block.line
                );
            } else if let Err(errors) = parse_result {
                let joined = errors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; ");
                panic!(
                    "unexpected parse failure in {doc}:{} => {joined}",
                    block.line
                );
            }
        }
    }
}

#[test]
fn metadata_reference_mentions_all_authoring_fields() {
    let path = user_docs_dir().join("metadata-reference.md");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

    for token in REQUIRED_METADATA_REFERENCE_TOKENS {
        assert!(
            content.contains(token),
            "metadata-reference.md missing field token {token}"
        );
    }
}

fn user_docs_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("user")
}

fn extract_yaml_blocks(markdown: &str) -> Vec<YamlBlock> {
    let mut blocks = Vec::new();
    let mut in_yaml = false;
    let mut start_line = 0usize;
    let mut current = Vec::<String>::new();

    for (index, line) in markdown.lines().enumerate() {
        let line_number = index + 1;
        let trimmed = line.trim();

        if !in_yaml {
            if trimmed.starts_with("```yaml") {
                in_yaml = true;
                start_line = line_number;
                current.clear();
            }
            continue;
        }

        if trimmed == "```" {
            let (content, expected_error) = normalize_yaml_block(&current);
            blocks.push(YamlBlock {
                content,
                expected_error,
                line: start_line,
            });
            in_yaml = false;
            continue;
        }

        current.push(line.to_string());
    }

    blocks
}

fn normalize_yaml_block(lines: &[String]) -> (String, Option<String>) {
    let mut expected_error = None;
    let mut body = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if expected_error.is_none() && trimmed.starts_with("# expected-error:") {
            let code = trimmed
                .trim_start_matches("# expected-error:")
                .trim()
                .to_string();
            expected_error = Some(code);
            continue;
        }

        body.push(line.clone());
    }

    let content = if body.is_empty() {
        String::new()
    } else {
        format!("{}\n", body.join("\n"))
    };

    (content, expected_error)
}

trait UnwrapErrOrElse<T, E> {
    fn unwrap_err_or_else<F>(self, f: F) -> E
    where
        F: FnOnce(T) -> E;
}

impl<T, E> UnwrapErrOrElse<T, E> for Result<T, E> {
    fn unwrap_err_or_else<F>(self, f: F) -> E
    where
        F: FnOnce(T) -> E,
    {
        match self {
            Ok(value) => f(value),
            Err(err) => err,
        }
    }
}
