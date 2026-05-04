use std::collections::{HashMap, HashSet};

use cargo_metadata::{Metadata, MetadataCommand, PackageId};

/// # Panics
/// Panics if `cargo metadata` cannot be executed against the workspace.
pub fn workspace_metadata() -> Metadata {
    MetadataCommand::new()
        .no_deps()
        .exec()
        .expect("cargo metadata failed — is the workspace valid?")
}

/// # Panics
/// Panics if `cargo metadata` cannot be executed against the workspace.
pub fn resolved_metadata() -> Metadata {
    MetadataCommand::new()
        .exec()
        .expect("cargo metadata --resolve failed")
}

pub fn workspace_package_names(metadata: &Metadata) -> HashSet<String> {
    metadata
        .workspace_members
        .iter()
        .filter_map(|id| metadata.packages.iter().find(|p| &p.id == id))
        .map(|p| p.name.clone())
        .collect()
}

pub fn generator_crate_names(metadata: &Metadata) -> Vec<String> {
    workspace_package_names(metadata)
        .into_iter()
        .filter(|name| name.starts_with("know_now_gen_"))
        .collect()
}

/// # Panics
/// Panics if the metadata was produced without `--resolve`.
pub fn transitive_deps(metadata: &Metadata, package_name: &str) -> HashSet<String> {
    let resolve = metadata.resolve.as_ref().expect("resolve graph missing");

    let id_to_name: HashMap<&PackageId, &str> = metadata
        .packages
        .iter()
        .map(|p| (&p.id, p.name.as_str()))
        .collect();

    let name_to_id: HashMap<&str, &PackageId> = metadata
        .packages
        .iter()
        .map(|p| (p.name.as_str(), &p.id))
        .collect();

    let Some(start_id) = name_to_id.get(package_name) else {
        return HashSet::new();
    };

    let adj: HashMap<&PackageId, Vec<&PackageId>> = resolve
        .nodes
        .iter()
        .map(|node| (&node.id, node.deps.iter().map(|d| &d.pkg).collect()))
        .collect();

    let mut visited = HashSet::new();
    let mut stack = vec![*start_id];

    while let Some(id) = stack.pop() {
        if !visited.insert(id) {
            continue;
        }
        if let Some(neighbors) = adj.get(id) {
            for neighbor in neighbors {
                if !visited.contains(*neighbor) {
                    stack.push(neighbor);
                }
            }
        }
    }

    visited
        .into_iter()
        .filter_map(|id| id_to_name.get(id).map(|s| (*s).to_owned()))
        .filter(|name| name != package_name)
        .collect()
}

/// # Panics
/// Panics if the metadata was produced without `--resolve`.
pub fn direct_deps(metadata: &Metadata, package_name: &str) -> HashSet<String> {
    let resolve = metadata.resolve.as_ref().expect("resolve graph missing");

    let id_to_name: HashMap<&PackageId, &str> = metadata
        .packages
        .iter()
        .map(|p| (&p.id, p.name.as_str()))
        .collect();

    let name_to_id: HashMap<&str, &PackageId> = metadata
        .packages
        .iter()
        .map(|p| (p.name.as_str(), &p.id))
        .collect();

    let Some(start_id) = name_to_id.get(package_name) else {
        return HashSet::new();
    };

    resolve
        .nodes
        .iter()
        .find(|node| &node.id == *start_id)
        .map(|node| {
            node.deps
                .iter()
                .filter_map(|d| id_to_name.get(&d.pkg).map(|s| (*s).to_owned()))
                .collect()
        })
        .unwrap_or_default()
}

pub const BANNED_YAML_CRATES: &[&str] = &[
    "serde_yaml",
    "serde_yml",
    "serde-saphyr",
    "marked-yaml",
    "saphyr-parser",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_control_transitive_deps_detects_known_dep() {
        let metadata = resolved_metadata();
        let fitness_deps = transitive_deps(&metadata, "know_now_fitness");
        assert!(
            fitness_deps.contains("serde_json"),
            "negative control failed: know_now_fitness should transitively \
             depend on serde_json (via cargo_metadata)"
        );
    }

    #[test]
    fn negative_control_generator_names_match_prefix() {
        let metadata = workspace_metadata();
        let generators = generator_crate_names(&metadata);
        for gen in &generators {
            assert!(
                gen.starts_with("know_now_gen_"),
                "generator name {gen} does not match prefix"
            );
        }
        assert!(
            !generators.is_empty(),
            "negative control: expected at least one generator crate in workspace"
        );
    }

    #[test]
    fn negative_control_banned_yaml_list_is_nonempty() {
        assert!(
            !BANNED_YAML_CRATES.is_empty(),
            "negative control: BANNED_YAML_CRATES must not be empty"
        );
    }
}
