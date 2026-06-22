#!/usr/bin/env bash
set -euo pipefail

# ------------------------------------------------------------------
# validate.sh — Run all project checks from the repo root.
#
# Usage:
#   ./tools/validate.sh              # run all checks
#   ./tools/validate.sh rust         # backend Rust checks only
#   ./tools/validate.sh frontend     # frontend checks only
#   ./tools/validate.sh catalog      # catalog drift + tests only
# ------------------------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RUST_DIR="$REPO_ROOT/app/src-tauri"
APP_DIR="$REPO_ROOT/app"
CATALOG_DIFF_DIR="$REPO_ROOT/tools/catalog-diff"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

pass_count=0
fail_count=0

step() {
    local label="$1"
    shift
    echo ""
    echo -e "${YELLOW}[$label]${NC} $*"
}

pass() {
    echo -e "  ${GREEN}✓ PASS${NC}"
    pass_count=$((pass_count + 1))
}

fail() {
    echo -e "  ${RED}✗ FAIL${NC} $*"
    fail_count=$((fail_count + 1))
}

# ────────────────────────────
# Rust backend checks
# ────────────────────────────
rust_checks() {
    step "rust" "Format check (cargo fmt --check)"
    if cargo fmt --check --manifest-path "$RUST_DIR/Cargo.toml" 2>&1; then
        pass
    else
        fail "Run 'cargo fmt --manifest-path $RUST_DIR/Cargo.toml' to fix formatting"
    fi

    step "rust" "Lint (cargo clippy -- -D warnings)"
    if cargo clippy --manifest-path "$RUST_DIR/Cargo.toml" -- -D warnings 2>&1; then
        pass
    else
        fail "Fix clippy warnings before committing"
    fi

    step "rust" "Tests (cargo test)"
    if cargo test --manifest-path "$RUST_DIR/Cargo.toml" 2>&1; then
        pass
    else
        fail "Failing tests — run 'cargo test --manifest-path $RUST_DIR/Cargo.toml' for details"
    fi
}

# ────────────────────────────
# Frontend checks
# ────────────────────────────
frontend_checks() {
    step "frontend" "TypeScript typecheck (tsc --noEmit)"
    if npm --prefix "$APP_DIR" run typecheck 2>&1; then
        pass
    else
        fail "TypeScript errors — fix before committing"
    fi

    step "frontend" "Unit tests (vitest)"
    if npm --prefix "$APP_DIR" run test -- --run 2>&1; then
        pass
    else
        fail "Frontend test failures — run 'npm --prefix app run test -- --run' for details"
    fi

    step "frontend" "Build (vite build)"
    if npm --prefix "$APP_DIR" run build 2>&1; then
        pass
    else
        fail "Frontend build failed — run 'npm --prefix app run build' for details"
    fi
}

# ────────────────────────────
# Catalog tooling checks
# ────────────────────────────
catalog_checks() {
    step "catalog" "Drift detection (catalog_diff.py)"
    if python3 "$CATALOG_DIFF_DIR/catalog_diff.py" 2>&1; then
        pass
    else
        fail "Catalog drift detected or drift check failed"
    fi

    step "catalog" "Python tests (wiki_parser.py)"
    if python3 "$CATALOG_DIFF_DIR/tests/test_wiki_parser.py" 2>&1; then
        pass
    else
        fail "Catalog wiki parser tests failed"
    fi

    step "catalog" "Python tests (catalog_update.py)"
    if python3 "$CATALOG_DIFF_DIR/tests/test_catalog_update.py" 2>&1; then
        pass
    else
        fail "Catalog update tests failed"
    fi
}

# ────────────────────────────
# Main
# ────────────────────────────
all_checks() {
    rust_checks
    frontend_checks
    catalog_checks
}

echo "============================================="
echo "  Stellaris Hunter — Validation Suite"
echo "============================================="
echo "  Repo root: $REPO_ROOT"
echo "  Rust dir:  $RUST_DIR"
echo "  App dir:   $APP_DIR"

case "${1:-all}" in
    all)
        all_checks
        ;;
    rust)
        rust_checks
        ;;
    frontend)
        frontend_checks
        ;;
    catalog)
        catalog_checks
        ;;
    *)
        echo "Usage: $0 [all|rust|frontend|catalog]" >&2
        exit 2
        ;;
esac

echo ""
echo "============================================="
echo -e "  ${GREEN}${pass_count} passed${NC}  ${RED}${fail_count} failed${NC}"
echo "============================================="

if [ "$fail_count" -gt 0 ]; then
    exit 1
fi
