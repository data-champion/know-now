#!/usr/bin/env bash
set -euo pipefail

echo "[phase-1] Running Phase 1 E2E integration tests"

echo "[phase-1] step 1: determinism tests"
cargo test --package know_now_core --test determinism -- --nocapture

echo "[phase-1] step 2: pipeline + writer + volatile-state tests"
cargo test --package know_now_core --test phase1_e2e -- --nocapture

echo "[phase-1] All Phase 1 E2E tests passed"
