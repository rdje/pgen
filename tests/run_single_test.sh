#!/bin/bash
# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Change to the parent directory where run_parser.pl is located
cd "$SCRIPT_DIR/.."
perl run_parser.pl "$1" "$2" 