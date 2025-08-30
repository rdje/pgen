#!/bin/bash

# Documentation Update Helper Script
# This script helps ensure documentation stays in sync with code changes

echo "=== LinkedSpec Documentation Update Helper ==="
echo ""

# Check if generate_test_input.pl has been modified
if [ -f "generate_test_input.pl" ]; then
    echo "✅ generate_test_input.pl found"
    
    # Check if documentation exists
    if [ -f "GENERATE_TEST_INPUT.md" ]; then
        echo "✅ GENERATE_TEST_INPUT.md found"
        
        # Get modification times
        SCRIPT_TIME=$(stat -f "%m" generate_test_input.pl 2>/dev/null || stat -c "%Y" generate_test_input.pl 2>/dev/null)
        DOC_TIME=$(stat -f "%m" GENERATE_TEST_INPUT.md 2>/dev/null || stat -c "%Y" GENERATE_TEST_INPUT.md 2>/dev/null)
        
        if [ "$SCRIPT_TIME" -gt "$DOC_TIME" ]; then
            echo "⚠️  WARNING: generate_test_input.pl is newer than GENERATE_TEST_INPUT.md"
            echo "   Please update the documentation to reflect any changes."
            echo ""
            echo "   To update documentation:"
            echo "   1. Review changes in generate_test_input.pl"
            echo "   2. Update GENERATE_TEST_INPUT.md accordingly"
            echo "   3. Run this script again to verify"
        else
            echo "✅ Documentation is up to date"
        fi
    else
        echo "❌ GENERATE_TEST_INPUT.md not found"
        echo "   Please create documentation for generate_test_input.pl"
    fi
else
    echo "❌ generate_test_input.pl not found"
fi

echo ""
echo "=== Documentation Files ==="
echo "📄 TEST_GUIDE.md - Main test framework documentation"
echo "📄 GENERATE_TEST_INPUT.md - Test input generator documentation"
echo ""
echo "=== Quick Commands ==="
echo "📖 View generator docs: cat GENERATE_TEST_INPUT.md"
echo "📖 View test guide: cat TEST_GUIDE.md"
echo "🔍 Check script help: perl generate_test_input.pl --help"
echo "🧪 Test generator: perl generate_test_input.pl specs/valid/basic.spec"





