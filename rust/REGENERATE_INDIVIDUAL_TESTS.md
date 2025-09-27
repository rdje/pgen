# How to Regenerate Individual Tests

## Quick Command

```bash
cargo run --bin sync_tests sync
```

This single command will:
- ✅ Regenerate `src/individual_tests.rs` with correct format strings
- ✅ Update Makefile targets  
- ✅ Update test registry
- ✅ Fix any format string compilation errors in generated tests

## Why This is Needed

The `individual_tests.rs` file is **auto-generated** and should **never be manually edited**. When format string errors occur in this file, the solution is to:

1. **Fix the generator** (in `src/individual_tests_generator.rs`) 
2. **Regenerate the file** using the sync command above

## Background Context

This file is part of the **PGEN Test Automation System** that provides automatic synchronization between stress test files and generated artifacts.

## Full Documentation

For complete details, see:
- [`docs/TEST_AUTOMATION.md`](docs/TEST_AUTOMATION.md) - Complete test automation system documentation
- [`docs/DEVELOPMENT_GUIDE.md`](docs/DEVELOPMENT_GUIDE.md) - Development and extension guide

## Alternative Commands

If the basic sync command doesn't work:

```bash
# Force complete synchronization
cargo run --bin sync_tests sync

# Check if sync is needed first
cargo run --bin sync_tests check

# Quick regeneration (faster, existing registry only)  
cargo run --bin sync_tests quick-sync
```

## Generator Source

The actual generator logic is in:
- `src/individual_tests_generator.rs` - Main test generation logic
- `src/test_automation.rs` - Orchestrates the full sync process

## Remember

**DO NOT manually edit `src/individual_tests.rs`** - it will be overwritten on the next sync!