#!/usr/bin/env python3
import json
import glob
import re

def fix_roundtrip_file(filepath):
    with open(filepath, 'r') as f:
        data = json.load(f)
    
    # If it's an array of test objects
    if isinstance(data, list):
        for test in data:
            if 'input' in test and 'expected_round_trip' in test:
                # For round-trip testing, expected should match input
                test['expected_round_trip'] = test['input']
    
    # If it's an object with a tests array
    elif isinstance(data, dict) and 'tests' in data:
        for test in data['tests']:
            if 'input' in test and 'expected_round_trip' in test:
                test['expected_round_trip'] = test['input']
    
    with open(filepath, 'w') as f:
        json.dump(data, f, indent=2)

# Find all JSON files in test_data
for filepath in glob.glob('rust/test_data/**/*.json'):
    print(f"Fixing {filepath}")
    fix_roundtrip_file(filepath)

print("All round-trip test files fixed!")
