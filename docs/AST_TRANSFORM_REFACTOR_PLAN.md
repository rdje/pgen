# AST Transform Refactoring Plan

## Current Architecture
```
AST::Transform.pm:
  process_transformation_phases() → step6_generate_parser_code() → Parser Code
```

## Target Architecture
```
AST::Transform.pm:
  process_transformation_phases() → Final AST → [CodeGenerator.pm | DataGenerator.pm]
```

## Refactoring Steps

### Step 1: Extract Core Pipeline
Create a function that stops at step 5:
```perl
sub process_to_final_ast {
    my ($input, %options) = @_;
    
    # Existing logic through step 5
    my $raw_ast = (ref($input) eq 'ARRAY') ? $input : load_ebnf_spec_from_content($input);
    my $step2_result = step2_group_by_or($raw_ast);
    my $step2_5_result = step2_5_handle_parentheses($step2_result);
    my $step3_result = step3_parse_sequences($step2_5_result);
    my $step4_result = step4_handle_quantifiers($step3_result);
    my ($final_ast, $rule_order) = step5_build_tree_structure($step4_result);
    
    return ($final_ast, $rule_order);
}
```

### Step 2: Update Existing Code Generation
```perl
sub process_transformation_phases {
    my ($input, %options) = @_;
    
    # Get final AST
    my ($final_ast, $rule_order) = process_to_final_ast($input, %options);
    
    # Generate parser code (existing step 6)
    my $step6_result = step6_generate_parser_code($final_ast, $rule_order);
    
    return $step6_result;
}
```

### Step 3: Create Data Generator Module
```perl
# New file: perl/AST/DataGenerator.pm
package AST::DataGenerator;

sub generate_test_data {
    my ($final_ast, $rule_order, %options) = @_;
    
    # Generate pseudo-random text from final AST
    # (This is where our data generation logic goes)
}
```

### Step 4: Create Unified CLI Tools
```perl
# tools/generate_test_data.pl
use AST::Transform qw(process_to_final_ast);
use AST::DataGenerator qw(generate_test_data);

# Parse EBNF → Final AST
my ($final_ast, $rule_order) = process_to_final_ast($ebnf_content);

# Final AST → Test Data
my $test_data = generate_test_data($final_ast, $rule_order, %options);
```

## Benefits

### 1. Clean Separation
- **AST::Transform**: Core grammar processing (JSON → Final AST)
- **AST::CodeGenerator**: Final AST → Parser code  
- **AST::DataGenerator**: Final AST → Test data

### 2. Perfect Reuse
- Left-recursion elimination works for both
- Quantified groups work for both
- Any future improvements benefit both

### 3. Consistent Grammar Interpretation
- Generator and parser use identical AST processing
- No divergence in grammar understanding
- Perfect round-trip validation

## Implementation Plan

### Phase 1: Extract Core (Non-Breaking)
1. Add `process_to_final_ast()` function to AST::Transform.pm
2. Keep existing `process_transformation_phases()` working
3. Test that existing functionality is unchanged

### Phase 2: Create DataGenerator
1. Create `perl/AST/DataGenerator.pm`
2. Implement text generation from final AST
3. Create basic CLI tool

### Phase 3: Advanced Features
1. Add probability weighting
2. Add recursion depth control
3. Add quantifier count control

This approach reuses ALL existing logic while enabling clean extensibility.
