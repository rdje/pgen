# CLI Reference - PGEN Test Automation System

> Historical note
> This document describes an older `sync_tests` CLI workflow that is not the primary
> current interface. The active user-facing CLI surface is centered on `ast_pipeline`
> and `test_runner`, documented in `README.md` and `PGEN_USER_GUIDE.md`.

**Complete Command-Line Tool Documentation**

---

## 📖 Overview

The PGEN Test Automation System provides two main command-line interfaces:

1. **`sync_tests`** - Main synchronization and management tool
2. **Make targets** - Integration with existing build system

This document provides comprehensive reference for all available commands, options, and usage patterns.

---

## 🛠️ sync_tests Command

The primary CLI tool for managing test synchronization.

### Basic Syntax

```bash
cargo run --bin sync_tests <COMMAND> [OPTIONS]
```

### Quick Reference

| Command | Purpose | Example |
|---------|---------|---------|
| `sync` | Full synchronization | `cargo run --bin sync_tests sync` |
| `quick-sync` | Regenerate without discovery | `cargo run --bin sync_tests quick-sync` |
| `check` | Check if sync needed | `cargo run --bin sync_tests check` |
| `stats` | Show test statistics | `cargo run --bin sync_tests stats` |
| `add` | Add new test case | `cargo run --bin sync_tests add return '$3' 'Test' scalar` |
| `remove` | Remove test case | `cargo run --bin sync_tests remove return '$3'` |
| `help` | Show help information | `cargo run --bin sync_tests help` |

---

## 📋 Detailed Command Reference

### **sync** / **full-sync**

Performs complete test discovery and synchronization.

```bash
cargo run --bin sync_tests sync
cargo run --bin sync_tests full-sync  # Alias
```

**What it does**:
1. Scans all `*_stress_test.rs` files for test cases
2. Updates the test registry with discovered tests
3. Regenerates Makefile targets
4. Updates `src/individual_tests.rs`
5. Synchronizes TestTargetMapper mappings

**Output**:
```
🔄 Starting full test synchronization...
Discovered 14 return test cases from src/comprehensive_stress_test.rs
Discovered 5 semantic test cases from src/semantic_annotation_stress_test.rs
💾 Saved updated test registry to: test_registry.json
Updated Makefile targets in: Makefile
Generated 19 individual test targets
Generated standalone Makefile: Makefile.stress
Generated individual tests file: src/individual_tests.rs
✅ Full test synchronization completed successfully!
```

**Exit Codes**:
- `0` - Success
- `1` - Error occurred

---

### **quick-sync**

Regenerates targets from existing registry without discovery.

```bash
cargo run --bin sync_tests quick-sync
```

**What it does**:
1. Loads existing test registry
2. Regenerates Makefile targets
3. Updates `src/individual_tests.rs`
4. Synchronizes TestTargetMapper mappings

**Use Cases**:
- Template changes that require regeneration
- Recovery from corrupted generated files
- Testing generation logic

**Performance**: ~50% faster than full sync since it skips discovery.

---

### **check**

Checks if synchronization is needed without performing it.

```bash
cargo run --bin sync_tests check
```

**Output Examples**:
```bash
# When sync is needed
🔄 Synchronization needed

# When up to date
✅ Tests are synchronized
```

**Exit Codes**:
- `0` - Tests are synchronized
- `1` - Error occurred
- `2` - Synchronization needed (special code for automation)

**Automation Usage**:
```bash
if ! cargo run --bin sync_tests check; then
    if [ $? -eq 2 ]; then
        echo "Running automatic sync..."
        cargo run --bin sync_tests sync
    else
        echo "Error checking sync status"
        exit 1
    fi
fi
```

---

### **stats**

Displays comprehensive test statistics and system status.

```bash
cargo run --bin sync_tests stats
```

**Sample Output**:
```
📊 Test Registry Statistics
═══════════════════════════
Return parser tests:   14
Semantic parser tests: 5
Regex parser tests:    8
─────────────────────
Total test cases:      27
Last updated:          2025-09-26 18:25:21

🎯 Available Make targets:
  make test-all          # Run all tests
  make test-return       # Run return parser tests
  make test-semantic     # Run semantic parser tests
  make test-quick        # Run quick test suite
  make test-stats        # Show test statistics

🦀 Rust test integration:
  cargo test             # Run all individual tests
  cargo test test_all_return_parser
  cargo test test_all_semantic_parser
```

**Information Displayed**:
- Test counts by parser type
- Total test cases
- Last synchronization time
- Available Make targets
- Rust test integration commands
- Registry status

---

### **add**

Adds a new test case to the registry.

```bash
cargo run --bin sync_tests add <parser_type> <input> <description> [category]
```

**Parameters**:
- `parser_type` - Type of parser (`return`, `semantic`, `regex`)
- `input` - Test input string (quoted if contains spaces)
- `description` - Human-readable description
- `category` - Optional category (auto-detected if not provided)

**Examples**:
```bash
# Add a return parser test
cargo run --bin sync_tests add return '$3' 'Third scalar reference' scalar

# Add a semantic parser test  
cargo run --bin sync_tests add semantic '@priority: high' 'Priority annotation' priority

# Add with auto-detected category
cargo run --bin sync_tests add return '[1, 2, 3]' 'Number array'

# Complex input with escaping
cargo run --bin sync_tests add return '$obj.prop[0]' 'Complex accessor chain' accessor
```

**Behavior**:
- Automatically categorizes if category not provided
- Performs immediate sync after addition
- Validates input format
- Prevents duplicate test cases

---

### **remove**

Removes a test case from the registry.

```bash
cargo run --bin sync_tests remove <parser_type> <input>
```

**Parameters**:
- `parser_type` - Type of parser (`return`, `semantic`, `regex`)
- `input` - Exact test input string to remove

**Examples**:
```bash
# Remove a specific test case
cargo run --bin sync_tests remove return '$3'

# Remove semantic test
cargo run --bin sync_tests remove semantic '@priority: high'
```

**Exit Codes**:
- `0` - Test case removed successfully
- `1` - Error occurred or test case not found

**Output**:
```bash
# Success
✅ Test case removed successfully

# Not found
⚠️  Test case not found
```

---

### **help**

Shows comprehensive help information.

```bash
cargo run --bin sync_tests help
cargo run --bin sync_tests --help  # Alias
cargo run --bin sync_tests -h      # Short alias
```

**Output Sections**:
- Command descriptions
- Usage examples
- Exit code explanations
- Common workflows

---

## 🎯 Make Target Reference

Integration with the existing Make-based build system.

### Sync Management Targets

```bash
# Status and checking
make sync-status              # Show detailed sync status
make check-sync-needed        # Check and auto-sync if needed

# Manual synchronization  
make force-sync               # Force complete synchronization
make auto-sync               # Internal auto-sync (used by system)

# Maintenance
make clean-sync              # Clean all sync state
make emergency-sync          # Recovery sync if tools broken
```

### File Watching Targets

```bash
# Real-time monitoring (requires fswatch on macOS)
make watch-sync              # Watch files for changes
brew install fswatch         # Install fswatch first

# Polling fallback
make poll-sync               # Poll files every 2 seconds
```

### Development Integration

All existing test targets automatically get sync checking:

```bash
# These will auto-sync if needed before running
make test-return-scalar-1      # Individual test case
make return_parser            # Parser comprehensive flow
make semantic_parser          # Parser comprehensive flow  
make test-parser              # Generated parser testing
```

---

## 🔧 Advanced Usage Patterns

### **Batch Operations**

Add multiple test cases efficiently:

```bash
# Using a loop
for input in '$1' '$2' '$10'; do
    cargo run --bin sync_tests add return "$input" "Scalar reference" scalar
done

# From a file
while IFS= read -r input; do
    cargo run --bin sync_tests add return "$input" "Batch test"
done < test_inputs.txt
```

### **Integration with CI/CD**

```bash
#!/bin/bash
# CI/CD integration script

# Check if sync is needed
if ! cargo run --bin sync_tests check; then
    if [ $? -eq 2 ]; then
        echo "::warning::Test synchronization needed in CI"
        cargo run --bin sync_tests sync
        
        # Commit generated files if needed
        if [ -n "$(git status --porcelain)" ]; then
            git add test_registry.json Makefile.stress src/individual_tests.rs
            git commit -m "Auto-sync: Update generated test files"
        fi
    else
        echo "::error::Failed to check sync status"
        exit 1
    fi
fi
```

### **Development Workflow Automation**

```bash
#!/bin/bash
# Pre-commit hook

# Ensure tests are synchronized
make check-sync-needed

# Run affected tests
if git diff --cached --name-only | grep -q "_stress_test.rs"; then
    echo "Stress test files modified, running full test suite"
    make test-all
fi
```

### **Debugging and Troubleshooting**

```bash
# Enable verbose logging
export RUST_LOG=debug
cargo run --bin sync_tests sync

# Debug specific issues
export SYNC_DEBUG=1
make check-sync-needed

# Recovery procedures
make clean-sync
make emergency-sync
cargo run --bin sync_tests sync
```

---

## 🔍 Exit Code Reference

Understanding command exit codes for automation:

| Exit Code | Meaning | Commands |
|-----------|---------|----------|
| `0` | Success | All commands |
| `1` | General error | All commands |
| `2` | Sync needed | `check` command only |

### **Exit Code Usage Examples**

```bash
# Simple success check
if cargo run --bin sync_tests sync; then
    echo "Sync completed successfully"
else
    echo "Sync failed"
    exit 1
fi

# Handle different exit codes
cargo run --bin sync_tests check
case $? in
    0) echo "Already synchronized" ;;
    1) echo "Error occurred"; exit 1 ;;
    2) echo "Sync needed"; make force-sync ;;
esac
```

---

## 📊 Output Format Reference

### **Standard Output Formats**

All tools use consistent emoji-based status indicators:

| Symbol | Meaning | Context |
|--------|---------|---------|
| 🔄 | In progress | Operations starting |
| ✅ | Success | Completed operations |
| ❌ | Error | Failed operations |
| ⚠️ | Warning | Non-fatal issues |
| 📊 | Information | Statistics/status |
| 🎯 | Action items | Usage suggestions |
| 💾 | File operations | File saves/updates |
| 🧪 | Testing | Test execution |

### **Structured Output**

For programmatic parsing, enable structured output:

```bash
# JSON output mode (if implemented)
cargo run --bin sync_tests stats --format=json

# Machine-readable output
cargo run --bin sync_tests check --quiet
echo $?  # Just the exit code
```

---

## 🔐 Environment Variables

Configure behavior through environment variables:

### **Core Configuration**

```bash
# Override monitored files
export STRESS_TEST_FILES="src/my_test.rs src/other_test.rs"

# Change sync tool command
export SYNC_TOOL="./my_sync_tool"

# Override registry location
export REGISTRY_FILE="./my_registry.json"

# Enable debug logging
export SYNC_DEBUG=1
export RUST_LOG=debug
```

### **Development Settings**

```bash
# Skip certain validations (development only)
export PGEN_DEV_MODE=1

# Override temporary directories
export TMPDIR="/custom/temp/dir"

# Customize output colors (if supported)
export NO_COLOR=1              # Disable colors
export FORCE_COLOR=1           # Force colors
```

---

## 🚨 Error Handling Reference

### **Common Error Patterns**

**Compilation Errors**:
```bash
$ cargo run --bin sync_tests sync
error: could not compile `pgen` (lib) due to 1 previous error
```
**Solution**: Fix compilation errors first, then run sync.

**Permission Errors**:
```bash
$ cargo run --bin sync_tests sync
Error: Permission denied (os error 13)
```
**Solution**: Check file permissions on registry and generated files.

**Missing Dependencies**:
```bash
$ make watch-sync
❌ No file watching tool found
   Install fswatch: brew install fswatch
```
**Solution**: Install required dependencies as suggested.

### **Recovery Procedures**

**Complete Reset**:
```bash
# Nuclear option - reset everything
make clean-sync
rm -f test_registry.json
rm -f src/individual_tests.rs
cargo run --bin sync_tests sync
```

**Partial Recovery**:
```bash
# Reset just generated files
rm -f Makefile.stress src/individual_tests.rs
cargo run --bin sync_tests quick-sync
```

**Registry Corruption**:
```bash
# Rebuild registry from source files
rm -f test_registry.json
cargo run --bin sync_tests sync
```

---

## 📚 Integration Examples

### **Git Hooks Integration**

**Pre-commit hook**:
```bash
#!/bin/sh
# .git/hooks/pre-commit

# Check if stress test files are being committed
if git diff --cached --name-only | grep -q "_stress_test.rs"; then
    echo "Stress test files modified, checking sync status..."
    
    if ! cargo run --bin sync_tests check --quiet; then
        echo "Error: Generated files are out of sync"
        echo "Run: cargo run --bin sync_tests sync"
        exit 1
    fi
fi
```

**Post-merge hook**:
```bash
#!/bin/sh
# .git/hooks/post-merge

# Auto-sync after merge if stress test files changed
if [ -n "$(git diff-tree -r --name-only HEAD~1 HEAD | grep "_stress_test.rs")" ]; then
    echo "Stress test files changed during merge, auto-syncing..."
    cargo run --bin sync_tests sync
fi
```

### **IDE Integration**

**VS Code Task** (`.vscode/tasks.json`):
```json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "Sync Tests",
            "type": "shell",
            "command": "cargo",
            "args": ["run", "--bin", "sync_tests", "sync"],
            "group": "build",
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": false,
                "panel": "shared"
            }
        }
    ]
}
```

### **Makefile Integration Examples**

**Custom targets**:
```makefile
# Custom development targets
.PHONY: dev-sync test-modified

dev-sync: check-sync-needed
	@echo "Development environment synchronized"

test-modified:
	@# Run tests for recently modified files
	@git diff --name-only HEAD~1 | grep "_stress_test.rs" | while read file; do \
		echo "Testing modifications in $$file"; \
		make check-sync-needed; \
	done
```

---

This CLI reference provides comprehensive documentation for all command-line interfaces in the PGEN Test Automation System. Use this reference to integrate the system into your development workflow, CI/CD pipelines, and automation scripts.
