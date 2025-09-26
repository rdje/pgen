#!/bin/bash
# Comprehensive Test Automation Setup Script
# Sets up complete automatic synchronization environment

set -e  # Exit on error

echo "🚀 PGEN Test Automation Setup"
echo "═══════════════════════════════"
echo ""

# Check dependencies
echo "1️⃣ Checking dependencies..."

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust and Cargo first."
    exit 1
fi
echo "   ✅ Cargo found"

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]] || [[ ! -f "src/lib.rs" ]]; then
    echo "❌ Not in the correct directory. Please run from the rust project root."
    exit 1
fi
echo "   ✅ In correct directory"

# Build the project to ensure sync tools are available
echo ""
echo "2️⃣ Building project with sync tools..."
cargo build --bin sync_tests --quiet
echo "   ✅ Sync tools built successfully"

# Perform initial synchronization
echo ""
echo "3️⃣ Performing initial test synchronization..."
if cargo run --bin sync_tests sync --quiet; then
    echo "   ✅ Initial sync completed"
else
    echo "   ⚠️  Initial sync had issues, but continuing..."
fi

# Show current sync status
echo ""
echo "4️⃣ Current synchronization status:"
make sync-status 2>/dev/null || {
    echo "   ℹ️  Sync status not available yet"
}

# Check if fswatch is available for file watching
echo ""
echo "5️⃣ Checking file watching capabilities..."
if command -v fswatch &> /dev/null; then
    echo "   ✅ fswatch found - real-time file watching available"
    echo "   💡 Use 'make watch-sync' for automatic sync on file changes"
else
    echo "   ⚠️  fswatch not found - only timestamp-based sync available"
    echo "   💡 Install with: brew install fswatch (macOS) or apt install inotify-tools (Linux)"
    echo "   💡 Alternative: use 'make poll-sync' for polling-based watching"
fi

# Test the automation
echo ""
echo "6️⃣ Testing automation integration..."

# Test a simple make target to ensure sync works
echo "   Testing 'make sync-status'..."
if make sync-status >/dev/null 2>&1; then
    echo "   ✅ Sync integration working"
else
    echo "   ⚠️  Sync integration may have issues"
fi

# Show generated files
echo ""
echo "7️⃣ Generated files:"
echo "═══════════════════"

files_to_check=(
    "test_registry.json"
    "Makefile.auto-sync"
    ".last_sync_timestamp"
    "src/individual_tests.rs"
)

for file in "${files_to_check[@]}"; do
    if [[ -f "$file" ]]; then
        echo "   ✅ $file"
    else
        echo "   ❌ $file (missing)"
    fi
done

# Show auto-sync integration status
echo ""
echo "8️⃣ Auto-sync integration status:"
echo "═══════════════════════════════"

# Check if Makefile includes auto-sync
if grep -q "include Makefile.auto-sync" Makefile 2>/dev/null; then
    echo "   ✅ Makefile includes auto-sync"
else
    echo "   ⚠️  Makefile may not include auto-sync - check 'include Makefile.auto-sync' line"
fi

# Check if stress test files exist
stress_files=("src/comprehensive_stress_test.rs" "src/semantic_annotation_stress_test.rs")
for file in "${stress_files[@]}"; do
    if [[ -f "$file" ]]; then
        echo "   ✅ $file found"
    else
        echo "   ❌ $file missing"
    fi
done

echo ""
echo "9️⃣ Setup Summary & Usage"
echo "══════════════════════"
echo ""
echo "🎯 The test environment will now automatically synchronize whenever"
echo "   stress test files (*_stress_test.rs) are modified!"
echo ""
echo "💡 Key Features Enabled:"
echo "   • Automatic sync on file changes"
echo "   • Timestamp-based change detection"
echo "   • Integration with all Make targets"
echo "   • Individual test generation"
echo "   • Makefile target generation"
echo ""
echo "🛠️  Usage Examples:"
echo ""
echo "   # Any test command will auto-sync first:"
echo "   make test-return-scalar-1     # Auto-syncs if needed, then runs test"
echo "   make semantic_parser          # Auto-syncs if needed, then runs parser"
echo "   make return_parser            # Auto-syncs if needed, then runs tests"
echo ""
echo "   # Manual sync commands:"
echo "   make sync-status              # Show current sync status"
echo "   make force-sync               # Force synchronization"
echo "   make check-sync-needed        # Check and sync if needed"
echo ""
echo "   # File watching (requires fswatch):"
echo "   make watch-sync               # Watch files and auto-sync on changes"
echo "   make poll-sync                # Poll files every 2 seconds"
echo ""
echo "   # CLI sync tool:"
echo "   cargo run --bin sync_tests sync      # Manual sync"
echo "   cargo run --bin sync_tests stats     # Show test statistics"
echo "   cargo run --bin sync_tests check     # Check if sync needed"
echo ""
echo "🔧 Development Workflow:"
echo "   1. Edit stress test files (add/remove test inputs)"
echo "   2. Run any make command (auto-sync happens automatically)"
echo "   3. New tests are immediately available as Make targets"
echo "   4. Individual Rust tests are also auto-generated"
echo ""
echo "🎉 Test automation setup complete!"
echo "   Your development environment is now fully automated!"

# Final check - demonstrate the automation working
echo ""
echo "🧪 Testing Automation (Quick Demo):"
echo "  Running 'make sync-status' to show current state..."
echo ""
make sync-status

echo ""
echo "✨ Setup Complete! The test environment will now automatically"
echo "   adjust whenever you modify stress test files."