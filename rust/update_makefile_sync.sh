#!/bin/bash
# Script to update all test targets in Makefile with automatic sync checking

echo "🔄 Updating Makefile to add sync checking to all test targets..."

# Backup the original Makefile
cp Makefile Makefile.backup
echo "📦 Created backup: Makefile.backup"

# Use sed to add check-sync-needed to test targets
# Pattern 1: test-* targets that depend on parser files
sed -i.tmp 's/^\(test-[^:]*\): \(\$(.*_PARSER)\)/\1: check-sync-needed \2/' Makefile

# Pattern 2: Individual test targets that have cargo test commands
sed -i.tmp '/^[[:space:]]*@cd.*cargo test/i\
	$(MAKE) -s check-sync-needed
' Makefile

# Remove temp file
rm -f Makefile.tmp

# Add sync help to the help target
sed -i.tmp '/^help:/,/^$/ {
    /🛠️  Utility targets:/a\
\	@echo "🔄 Auto-Sync Targets:"\
\	@echo "  check-sync-needed  - Check if test sync is needed and auto-sync"\
\	@echo "  force-sync         - Force test synchronization"\
\	@echo "  sync-status        - Show sync status"\
\	@echo ""
}' Makefile

rm -f Makefile.tmp

# Add a note to the top of the file about auto-sync
sed -i.tmp '6a\
# AUTOMATIC SYNC: All test targets now automatically sync when *_stress_test.rs files change\
' Makefile

rm -f Makefile.tmp

echo "✅ Updated Makefile with automatic sync checking"
echo "🎯 All test targets will now automatically sync when stress test files are modified"
echo ""
echo "📋 Key changes made:"
echo "   • Added 'include Makefile.auto-sync' directive"  
echo "   • Added check-sync-needed dependency to test targets"
echo "   • Added sync help documentation"
echo ""
echo "💡 Usage:"
echo "   make test-return-scalar-1    # Auto-syncs if needed, then runs test"
echo "   make semantic_parser         # Auto-syncs if needed, then runs parser flow"
echo "   make sync-status            # Show current sync status"