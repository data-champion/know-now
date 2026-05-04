use know_now_fitness::{
    direct_deps, generator_crate_names, resolved_metadata, transitive_deps, workspace_metadata,
    workspace_package_names, BANNED_YAML_CRATES,
};
use std::sync::LazyLock;

use cargo_metadata::Metadata;

static RESOLVED: LazyLock<Metadata> = LazyLock::new(resolved_metadata);
static WORKSPACE: LazyLock<Metadata> = LazyLock::new(workspace_metadata);

// ---------------------------------------------------------------------------
// Invariant 1: Generator crates do not depend on YAML parser crates
// PRD §8.5, §8.7, §17.6 — generators consume the validated GeneratorContract,
// never raw YAML.
// ---------------------------------------------------------------------------
#[test]
fn inv01_generators_no_yaml_parser_deps() {
    let generators = generator_crate_names(&WORKSPACE);
    let mut violations = Vec::new();

    for gen in &generators {
        let deps = transitive_deps(&RESOLVED, gen);
        for banned in BANNED_YAML_CRATES {
            if deps.contains(*banned) {
                violations.push(format!("{gen} transitively depends on {banned}"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "INVARIANT 1 VIOLATED — generators must not depend on YAML parser crates \
         (AGENTS.md §4.1, PRD §8.5):\n  {}",
        violations.join("\n  ")
    );
}

// ---------------------------------------------------------------------------
// Invariant 2: YAML parser deps are isolated to know_now_metadata
// PRD §10.2, NFR-S13, NFR-M8
// ---------------------------------------------------------------------------
#[test]
fn inv02_yaml_parser_isolated_to_metadata() {
    let ws_names = workspace_package_names(&WORKSPACE);
    let mut violations = Vec::new();

    for name in &ws_names {
        if name == "know_now_metadata" || name == "know_now_fitness" {
            continue;
        }
        let deps = transitive_deps(&RESOLVED, name);
        for banned in BANNED_YAML_CRATES {
            if deps.contains(*banned) {
                violations.push(format!("{name} transitively depends on {banned}"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "INVARIANT 2 VIOLATED — YAML parser deps must be isolated to know_now_metadata \
         (AGENTS.md §4.1, PRD §10.2):\n  {}",
        violations.join("\n  ")
    );
}

// ---------------------------------------------------------------------------
// Invariant 3: Generator crates depend on contract/codegen/ir, never on
// know_now_metadata directly
// PRD §8.5, §8.7
// ---------------------------------------------------------------------------
#[test]
fn inv03_generators_no_direct_metadata_dep() {
    let generators = generator_crate_names(&WORKSPACE);
    let mut violations = Vec::new();

    for gen in &generators {
        let deps = transitive_deps(&RESOLVED, gen);
        if deps.contains("know_now_metadata") {
            violations.push(format!(
                "{gen} depends on know_now_metadata (should use know_now_contract/codegen/ir)"
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "INVARIANT 3 VIOLATED — generators must consume validated contract inputs, \
         not raw metadata (AGENTS.md §4.1, PRD §8.5):\n  {}",
        violations.join("\n  ")
    );
}

// ---------------------------------------------------------------------------
// Invariant 4: Generators depend on know_now_codegen (artifact-descriptor
// traits), not on know_now_writer internals
// PRD §9.3, §17.6
// ---------------------------------------------------------------------------
#[test]
fn inv04_generators_no_writer_dep() {
    let generators = generator_crate_names(&WORKSPACE);
    let mut violations = Vec::new();

    for gen in &generators {
        let deps = transitive_deps(&RESOLVED, gen);
        if deps.contains("know_now_writer") {
            violations.push(format!(
                "{gen} depends on know_now_writer (should use know_now_codegen)"
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "INVARIANT 4 VIOLATED — generators must return artifact descriptors via codegen traits, \
         not write files via know_now_writer (AGENTS.md §4.1, PRD §9.3):\n  {}",
        violations.join("\n  ")
    );
}

// ---------------------------------------------------------------------------
// Invariant 5: All writes go through the artifact writer
// Structural check: only know_now_writer (and know_now_core through it)
// should depend on std::fs write operations. This is partially enforced
// by invariants 4 and 12; full enforcement requires code-level review.
// ---------------------------------------------------------------------------
#[test]
fn inv05_writes_through_writer() {
    // Structural proxy: generators and templates must not depend on
    // know_now_writer (checked by inv04 and inv12). The positive half
    // (that know_now_core routes through know_now_writer) is a code-level
    // invariant verified when those crates gain implementation.
}

// ---------------------------------------------------------------------------
// Invariant 6: Artifact writer does not depend on generator crates
// know_now_writer must NOT have know_now_gen_* in its dep tree
// ---------------------------------------------------------------------------
#[test]
fn inv06_writer_no_generator_deps() {
    let generators = generator_crate_names(&WORKSPACE);
    let writer_deps = transitive_deps(&RESOLVED, "know_now_writer");
    let mut violations = Vec::new();

    for gen in &generators {
        if writer_deps.contains(gen) {
            violations.push(format!("know_now_writer transitively depends on {gen}"));
        }
    }

    assert!(
        violations.is_empty(),
        "INVARIANT 6 VIOLATED — know_now_writer must not depend on generator crates \
         (AGENTS.md §4.1):\n  {}",
        violations.join("\n  ")
    );
}

// ---------------------------------------------------------------------------
// Invariant 7: Diagnostics has no generator-specific dependencies
// know_now_diagnostics has no know_now_gen_* deps
// ---------------------------------------------------------------------------
#[test]
fn inv07_diagnostics_no_generator_deps() {
    let generators = generator_crate_names(&WORKSPACE);
    let diag_deps = transitive_deps(&RESOLVED, "know_now_diagnostics");
    let mut violations = Vec::new();

    for gen in &generators {
        if diag_deps.contains(gen) {
            violations.push(format!(
                "know_now_diagnostics transitively depends on {gen}"
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "INVARIANT 7 VIOLATED — know_now_diagnostics must be generator-agnostic \
         (NFR-M5):\n  {}",
        violations.join("\n  ")
    );
}

// ---------------------------------------------------------------------------
// Invariant 8: Lockfile resolution is isolated from artifact generation
// know_now_lock independent of know_now_gen_* and know_now_writer
// ---------------------------------------------------------------------------
#[test]
fn inv08_lock_isolated_from_generation() {
    let generators = generator_crate_names(&WORKSPACE);
    let lock_deps = transitive_deps(&RESOLVED, "know_now_lock");
    let mut violations = Vec::new();

    for gen in &generators {
        if lock_deps.contains(gen) {
            violations.push(format!("know_now_lock transitively depends on {gen}"));
        }
    }
    if lock_deps.contains("know_now_writer") {
        violations.push("know_now_lock transitively depends on know_now_writer".to_string());
    }

    assert!(
        violations.is_empty(),
        "INVARIANT 8 VIOLATED — lockfile resolution must be isolated from artifact generation:\n  {}",
        violations.join("\n  ")
    );
}

// ---------------------------------------------------------------------------
// Invariant 9: Policy validation cannot mutate metadata
// Code-level invariant: know_now_policy must not expose &mut access to
// ProjectGraph or Authoring* types. Verified when those types exist.
// Structural proxy: policy should not depend on know_now_writer.
// ---------------------------------------------------------------------------
#[test]
fn inv09_policy_no_writer_dep() {
    let policy_deps = transitive_deps(&RESOLVED, "know_now_policy");

    assert!(
        !policy_deps.contains("know_now_writer"),
        "INVARIANT 9 VIOLATED — know_now_policy must not depend on know_now_writer \
         (policy validation is read-only, PRD §14.4)"
    );
}

// ---------------------------------------------------------------------------
// Invariant 10: Local server write endpoints disabled unless explicitly enabled
// Code-level invariant verified when know_now_server gains implementation.
// Structural proxy: ensure the server crate exists.
// ---------------------------------------------------------------------------
#[test]
fn inv10_server_crate_exists() {
    let ws_names = workspace_package_names(&WORKSPACE);
    assert!(
        ws_names.contains("know_now_server"),
        "INVARIANT 10 — know_now_server crate must exist in workspace \
         (write-endpoint guard is verified at code level when implemented)"
    );
}

// ---------------------------------------------------------------------------
// Invariant 11: Dashboard/server API contract compatibility tested in CI
// Structural proxy: both know_now_server and know_now_contract exist.
// Full enforcement is a CI-level check wired in 42e.3.
// ---------------------------------------------------------------------------
#[test]
fn inv11_contract_crate_exists() {
    let ws_names = workspace_package_names(&WORKSPACE);
    assert!(
        ws_names.contains("know_now_contract"),
        "INVARIANT 11 — know_now_contract crate must exist (API compatibility \
         testing wired in CI)"
    );
}

// ---------------------------------------------------------------------------
// Invariant 12: Template packs cannot write files directly
// know_now_templates must not depend on know_now_writer
// ---------------------------------------------------------------------------
#[test]
fn inv12_templates_no_writer_dep() {
    let tmpl_deps = transitive_deps(&RESOLVED, "know_now_templates");

    assert!(
        !tmpl_deps.contains("know_now_writer"),
        "INVARIANT 12 VIOLATED — know_now_templates must not depend on know_now_writer \
         (templates return artifact descriptors, never write files directly; PRD §15.1.1)"
    );
}

// ---------------------------------------------------------------------------
// Invariant 13: know_now_templates produces artifact descriptors, does not
// bypass writer path-safety rules
// Structural: templates should depend on codegen (for descriptor traits),
// not on writer.
// ---------------------------------------------------------------------------
#[test]
fn inv13_templates_no_writer_bypass() {
    let tmpl_deps = transitive_deps(&RESOLVED, "know_now_templates");
    let generators = generator_crate_names(&WORKSPACE);
    let mut violations = Vec::new();

    if tmpl_deps.contains("know_now_writer") {
        violations.push("know_now_templates depends on know_now_writer".to_string());
    }
    for gen in &generators {
        if tmpl_deps.contains(gen) {
            violations.push(format!("know_now_templates depends on {gen}"));
        }
    }

    assert!(
        violations.is_empty(),
        "INVARIANT 13 VIOLATED — know_now_templates must not bypass the writer \
         or depend on generators (PRD §15.1.1, NFR-S17):\n  {}",
        violations.join("\n  ")
    );
}

// ---------------------------------------------------------------------------
// Invariant 14: Template rendering cannot access raw YAML parser types
// know_now_templates must not depend on YAML parser crates
// ---------------------------------------------------------------------------
#[test]
fn inv14_templates_no_yaml_parser() {
    let tmpl_deps = transitive_deps(&RESOLVED, "know_now_templates");
    let mut violations = Vec::new();

    for banned in BANNED_YAML_CRATES {
        if tmpl_deps.contains(*banned) {
            violations.push(format!(
                "know_now_templates transitively depends on {banned}"
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "INVARIANT 14 VIOLATED — template rendering must not access raw YAML parser types \
         (AGENTS.md §4.1):\n  {}",
        violations.join("\n  ")
    );
}

// ---------------------------------------------------------------------------
// Invariant 15: Custom template packs cannot register native MiniJinja
// functions, filters, tests, or loaders
// Compile-time API gate verified when know_now_templates gains implementation.
// Structural proxy: templates must not pull in unrestricted minijinja features.
// ---------------------------------------------------------------------------
#[test]
fn inv15_templates_restricted_minijinja() {
    // When know_now_templates adds minijinja, verify it does not enable
    // features that would allow custom native functions/filters/tests/loaders.
    // At stub stage, the invariant holds vacuously.
    let tmpl_deps = direct_deps(&RESOLVED, "know_now_templates");
    if !tmpl_deps.contains("minijinja") {
        return;
    }

    let packages = &RESOLVED.packages;
    let tmpl_pkg = packages.iter().find(|p| p.name == "know_now_templates");
    if let Some(pkg) = tmpl_pkg {
        for dep in &pkg.dependencies {
            if dep.name == "minijinja" {
                let features = &dep.features;
                let banned_features = ["custom_syntax", "loader", "fuel", "multi_template"];
                for banned in &banned_features {
                    assert!(
                        !features.iter().any(|f| f == *banned),
                        "INVARIANT 15 VIOLATED — know_now_templates must not enable \
                         minijinja feature `{banned}` (PRD §15.1, NFR-S22..S25)"
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Invariant 16: Unsupported renderer profile changes fail compatibility tests
// CI-level invariant: compatibility fixtures include renderer profile snapshots.
// Verified when compatibility fixture infrastructure exists (42e.3 / 42e.7).
// ---------------------------------------------------------------------------
#[test]
fn inv16_renderer_profile_compat() {
    // Placeholder: renderer profile compatibility is tested via fixture
    // snapshots in CI. The invariant here verifies the structural
    // prerequisite — know_now_templates exists in the workspace.
    let ws_names = workspace_package_names(&WORKSPACE);
    assert!(
        ws_names.contains("know_now_templates"),
        "INVARIANT 16 — know_now_templates crate must exist (renderer profile \
         compat testing is a CI fixture)"
    );
}

// ---------------------------------------------------------------------------
// Cross-cutting: no generator crate has a direct dependency on another
// generator crate (NFR-M1, NFR-SC2)
// ---------------------------------------------------------------------------
#[test]
fn generators_no_cross_deps() {
    let generators = generator_crate_names(&WORKSPACE);
    let mut violations = Vec::new();

    for gen in &generators {
        let deps = direct_deps(&RESOLVED, gen);
        for other in &generators {
            if gen != other && deps.contains(other) {
                violations.push(format!("{gen} directly depends on {other}"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Generator cross-dependency VIOLATED — adding a new generator must not \
         require modifying an existing one (NFR-M1, NFR-SC2):\n  {}",
        violations.join("\n  ")
    );
}

// ---------------------------------------------------------------------------
// Cross-cutting: know_now_diagnostics does not depend on know_now_metadata
// (diagnostics should be usable without pulling in the parser stack)
// ---------------------------------------------------------------------------
#[test]
fn diagnostics_no_metadata_dep() {
    let diag_deps = transitive_deps(&RESOLVED, "know_now_diagnostics");

    assert!(
        !diag_deps.contains("know_now_metadata"),
        "know_now_diagnostics must not depend on know_now_metadata — diagnostics \
         should be self-contained (PRD §8.1)"
    );
}
