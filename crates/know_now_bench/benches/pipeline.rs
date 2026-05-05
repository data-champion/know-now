#![allow(clippy::missing_docs_in_private_items)]

use std::path::PathBuf;
use std::process::Command;

use criterion::{criterion_group, criterion_main, Criterion};
use know_now_cli::commands::generate::{GenerateArgs, GenerateTarget, PruneMode};
use know_now_cli::commands::validate::ValidateArgs;
use know_now_cli::context::CommandContext;
use know_now_cli::output::OutputFormat;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("fixtures")
}

fn cli_binary() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target");

    if cfg!(debug_assertions) {
        path.push("debug");
    } else {
        path.push("release");
    }
    path.push("know-now");
    path
}

fn make_context(fixture: &str) -> CommandContext {
    CommandContext {
        format: OutputFormat::Text,
        verbose: false,
        debug: false,
        no_color: true,
        project_root: fixtures_dir().join(fixture),
        config_path: None,
    }
}

/// NFR-P1: CLI startup < 500 ms
fn bench_p1_cli_startup(c: &mut Criterion) {
    let binary = cli_binary();
    if !binary.exists() {
        eprintln!("Skipping P1: binary not found at {}", binary.display());
        return;
    }

    c.bench_function("p1_cli_startup", |b| {
        b.iter(|| {
            let output = Command::new(&binary)
                .arg("version")
                .output()
                .expect("failed to spawn CLI");
            assert!(output.status.success());
        });
    });
}

/// NFR-P2: Help output < 200 ms
fn bench_p2_help_output(c: &mut Criterion) {
    let binary = cli_binary();
    if !binary.exists() {
        eprintln!("Skipping P2: binary not found at {}", binary.display());
        return;
    }

    c.bench_function("p2_help_output", |b| {
        b.iter(|| {
            let output = Command::new(&binary)
                .arg("--help")
                .output()
                .expect("failed to spawn CLI");
            assert!(output.status.success());
        });
    });
}

/// NFR-P3: Validation 100 entities < 2 s
fn bench_p3_validate_100_entities(c: &mut Criterion) {
    let ctx = make_context("large_100_entity");

    c.bench_function("p3_validate_100_entities", |b| {
        b.iter(|| {
            let result = know_now_cli::commands::validate::run(&ctx, &ValidateArgs);
            assert!(result.is_ok(), "validation failed: {result:?}");
        });
    });
}

/// NFR-P4: Generation 10 entities < 5 s (dry-run measures full pipeline minus disk I/O)
fn bench_p4_generate_10_entities(c: &mut Criterion) {
    let ctx = make_context("demo_ecommerce");

    let args = GenerateArgs {
        target: Some(vec![GenerateTarget::All]),
        dry_run: true,
        strict: false,
        fail_on_warnings: false,
        locked: false,
        no_cache: true,
        changed: false,
        prune: PruneMode::None,
        accept_generated_overwrite: false,
        migration_safe: false,
    };

    c.bench_function("p4_generate_10_entities", |b| {
        b.iter(|| {
            let result = know_now_cli::commands::generate::run(&ctx, &args);
            assert!(result.is_ok(), "generation failed: {result:?}");
        });
    });
}

/// NFR-P5: Generation 100 entities < 60 s (dry-run)
fn bench_p5_generate_100_entities(c: &mut Criterion) {
    let ctx = make_context("large_100_entity");

    let args = GenerateArgs {
        target: Some(vec![GenerateTarget::All]),
        dry_run: true,
        strict: false,
        fail_on_warnings: false,
        locked: false,
        no_cache: true,
        changed: false,
        prune: PruneMode::None,
        accept_generated_overwrite: false,
        migration_safe: false,
    };

    let mut group = c.benchmark_group("p5_generate_100");
    group.sample_size(10);
    group.bench_function("p5_generate_100_entities", |b| {
        b.iter(|| {
            let result = know_now_cli::commands::generate::run(&ctx, &args);
            assert!(result.is_ok(), "generation failed: {result:?}");
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_p1_cli_startup,
    bench_p2_help_output,
    bench_p3_validate_100_entities,
    bench_p4_generate_10_entities,
    bench_p5_generate_100_entities,
);
criterion_main!(benches);
