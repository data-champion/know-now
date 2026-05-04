use std::path::{Path, PathBuf};

use minijinja::{Environment, UndefinedBehavior};
use serde::Serialize;

use crate::filters;
use crate::manifest::{Limits, PackManifest};

#[derive(Debug, Clone, Serialize)]
pub struct ArtifactDescriptor {
    pub path: String,
    pub content: String,
    pub generator: String,
    pub template: String,
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("template too large: {path} is {size} bytes (limit: {limit})")]
    TemplateTooLarge {
        path: String,
        size: usize,
        limit: usize,
    },
    #[error("too many templates: {count} (limit: {limit})")]
    TooManyTemplates { count: usize, limit: usize },
    #[error("output too large: {size} bytes (limit: {limit})")]
    OutputTooLarge { size: usize, limit: usize },
    #[error("too many output files: {count} (limit: {limit})")]
    TooManyOutputFiles { count: usize, limit: usize },
    #[error("render error in {template}: {source}")]
    Render {
        template: String,
        source: minijinja::Error,
    },
    #[error("template file read error: {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("include path escapes pack root: {path}")]
    IncludeEscape { path: String },
    #[error("dynamic include path rejected: {path}")]
    DynamicInclude { path: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct RenderReport {
    pub artifacts: Vec<ArtifactDescriptor>,
    pub templates_rendered: usize,
    pub total_output_bytes: usize,
}

/// # Errors
/// Returns errors on template limit violations, render failures, or I/O errors.
pub fn render_pack(
    pack_root: &Path,
    manifest: &PackManifest,
    context: &serde_json::Value,
) -> Result<RenderReport, RenderError> {
    let templates = discover_templates(pack_root, &manifest.limits)?;
    let artifacts = render_templates(pack_root, manifest, &templates, context)?;

    let total_output_bytes: usize = artifacts.iter().map(|a| a.content.len()).sum();
    if total_output_bytes > manifest.limits.max_output_bytes {
        return Err(RenderError::OutputTooLarge {
            size: total_output_bytes,
            limit: manifest.limits.max_output_bytes,
        });
    }

    if artifacts.len() > manifest.limits.max_output_files {
        return Err(RenderError::TooManyOutputFiles {
            count: artifacts.len(),
            limit: manifest.limits.max_output_files,
        });
    }

    Ok(RenderReport {
        templates_rendered: templates.len(),
        total_output_bytes,
        artifacts,
    })
}

fn discover_templates(
    pack_root: &Path,
    limits: &Limits,
) -> Result<Vec<PathBuf>, RenderError> {
    let templates_dir = pack_root.join("templates");
    if !templates_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut templates = Vec::new();
    collect_templates(&templates_dir, &mut templates);

    if templates.len() > limits.max_templates {
        return Err(RenderError::TooManyTemplates {
            count: templates.len(),
            limit: limits.max_templates,
        });
    }

    templates.sort();
    Ok(templates)
}

fn collect_templates(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_templates(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "j2" || ext == "jinja2") {
            out.push(path);
        }
    }
}

fn render_templates(
    pack_root: &Path,
    manifest: &PackManifest,
    templates: &[PathBuf],
    context: &serde_json::Value,
) -> Result<Vec<ArtifactDescriptor>, RenderError> {
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    env.set_fuel(Some(manifest.limits.max_fuel));
    filters::register_builtin_filters(&mut env);

    let mut artifacts = Vec::new();

    for template_path in templates {
        let source = std::fs::read_to_string(template_path).map_err(|e| RenderError::Io {
            path: template_path.display().to_string(),
            source: e,
        })?;

        if source.len() > manifest.limits.max_template_bytes {
            return Err(RenderError::TemplateTooLarge {
                path: template_path.display().to_string(),
                size: source.len(),
                limit: manifest.limits.max_template_bytes,
            });
        }

        validate_no_dynamic_includes(&source, template_path)?;
        validate_static_includes(&source, pack_root, template_path)?;

        let rel_path = template_path
            .strip_prefix(pack_root.join("templates"))
            .unwrap_or(template_path);
        let template_name = rel_path.display().to_string();

        env.add_template_owned(template_name.clone(), source)
            .map_err(|e| RenderError::Render {
                template: template_name.clone(),
                source: e,
            })?;

        let tmpl = env.get_template(&template_name).map_err(|e| RenderError::Render {
            template: template_name.clone(),
            source: e,
        })?;

        let rendered = tmpl.render(context).map_err(|e| RenderError::Render {
            template: template_name.clone(),
            source: e,
        })?;

        let output_filename = rel_path
            .with_extension("")
            .display()
            .to_string();

        let output_path = format!("{}/{output_filename}", manifest.output_dir);

        artifacts.push(ArtifactDescriptor {
            path: output_path,
            content: rendered,
            generator: format!("template:{}", manifest.name),
            template: template_name,
        });
    }

    Ok(artifacts)
}

fn validate_no_dynamic_includes(
    source: &str,
    template_path: &Path,
) -> Result<(), RenderError> {
    for line in source.lines() {
        if let Some(idx) = line.find("{% include") {
            let after = &line[idx..];
            if after.contains("~ ") || after.contains("+ ") {
                return Err(RenderError::DynamicInclude {
                    path: template_path.display().to_string(),
                });
            }
        }
    }
    Ok(())
}

fn validate_static_includes(
    source: &str,
    pack_root: &Path,
    template_path: &Path,
) -> Result<(), RenderError> {
    for line in source.lines() {
        if let Some(start) = line.find("{% include") {
            let after = &line[start..];
            if let Some(q_start) = after.find('"').or_else(|| after.find('\'')) {
                let quote_char = after.as_bytes()[q_start];
                let inner = &after[q_start + 1..];
                if let Some(q_end) = inner.find(quote_char as char) {
                    let include_path = &inner[..q_end];
                    let resolved = pack_root.join("templates").join(include_path);
                    let canonical = resolved
                        .canonicalize()
                        .unwrap_or_else(|_| resolved.clone());
                    let pack_templates = pack_root.join("templates");
                    if !canonical.starts_with(&pack_templates) {
                        return Err(RenderError::IncludeEscape {
                            path: template_path.display().to_string(),
                        });
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_pack(templates: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let tpl_dir = dir.path().join("templates");
        std::fs::create_dir_all(&tpl_dir).unwrap();
        for (name, content) in templates {
            let path = tpl_dir.join(name);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(path, content).unwrap();
        }
        dir
    }

    fn default_manifest() -> PackManifest {
        PackManifest {
            name: "test".into(),
            version: "1.0.0".into(),
            target: "postgres".into(),
            renderer: crate::manifest::RendererRef {
                kind: "know-now-minijinja".into(),
                profile: 1,
            },
            output_dir: "output".into(),
            permissions: crate::manifest::Permissions::default(),
            limits: crate::manifest::Limits::default(),
            trust: crate::manifest::TrustLevel::Untrusted,
            licensing: None,
        }
    }

    #[test]
    fn renders_simple_template() {
        let pack = setup_pack(&[("hello.sql.j2", "SELECT '{{ name }}';")]);
        let ctx = serde_json::json!({ "name": "world" });
        let report = render_pack(pack.path(), &default_manifest(), &ctx).unwrap();
        assert_eq!(report.artifacts.len(), 1);
        assert_eq!(report.artifacts[0].content, "SELECT 'world';");
        assert_eq!(report.artifacts[0].path, "output/hello.sql");
    }

    #[test]
    fn strict_undefined_fails() {
        let pack = setup_pack(&[("test.sql.j2", "{{ undefined_var }}")]);
        let ctx = serde_json::json!({});
        let result = render_pack(pack.path(), &default_manifest(), &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn fuel_exhaustion_fails() {
        let template = "{% for i in range(100000) %}x{% endfor %}";
        let pack = setup_pack(&[("exhaust.txt.j2", template)]);
        let mut manifest = default_manifest();
        manifest.limits.max_fuel = 100;
        let ctx = serde_json::json!({});
        let result = render_pack(pack.path(), &manifest, &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn template_size_limit_enforced() {
        let large = "x".repeat(300_000);
        let pack = setup_pack(&[("big.txt.j2", &large)]);
        let ctx = serde_json::json!({});
        let result = render_pack(pack.path(), &default_manifest(), &ctx);
        assert!(matches!(result, Err(RenderError::TemplateTooLarge { .. })));
    }

    #[test]
    fn template_count_limit_enforced() {
        let templates: Vec<(String, String)> = (0..110)
            .map(|i| (format!("t{i}.txt.j2"), "ok".into()))
            .collect();
        let refs: Vec<(&str, &str)> = templates
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let pack = setup_pack(&refs);
        let mut manifest = default_manifest();
        manifest.limits.max_templates = 100;
        let ctx = serde_json::json!({});
        let result = render_pack(pack.path(), &manifest, &ctx);
        assert!(matches!(result, Err(RenderError::TooManyTemplates { .. })));
    }

    #[test]
    fn output_size_limit_enforced() {
        let pack = setup_pack(&[("big.txt.j2", "{{ content }}")]);
        let mut manifest = default_manifest();
        manifest.limits.max_output_bytes = 10;
        let ctx = serde_json::json!({ "content": "a".repeat(100) });
        let result = render_pack(pack.path(), &manifest, &ctx);
        assert!(matches!(result, Err(RenderError::OutputTooLarge { .. })));
    }

    #[test]
    fn builtin_filters_available() {
        let pack = setup_pack(&[("test.txt.j2", "{{ name | snake_case }}")]);
        let ctx = serde_json::json!({ "name": "CustomerOrder" });
        let report = render_pack(pack.path(), &default_manifest(), &ctx).unwrap();
        assert_eq!(report.artifacts[0].content, "customer_order");
    }

    #[test]
    fn no_templates_dir_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = serde_json::json!({});
        let report = render_pack(dir.path(), &default_manifest(), &ctx).unwrap();
        assert!(report.artifacts.is_empty());
    }
}
