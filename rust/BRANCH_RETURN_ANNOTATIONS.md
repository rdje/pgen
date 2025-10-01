# Branch-Level Return Annotations Implementation Plan

## Problem Statement

Return annotations in EBNF are attached to **branches/alternatives**, not rules. However, the current Rust AST pipeline implementation extracts return annotations at the rule level during Stage 1, losing the association with specific branches.

## Current Flow (Incorrect)

1. **JSON Input**: Return annotations appear as tokens inline with grammar tokens
   ```json
   ["rule", "key"],
   ["regex", "\"([^\"]+)\""],
   ["return_scalar", "$1"],    // <-- Return annotation for first branch
   ["operator", "|"],
   ["regex", "([a-zA-Z_]\\w*)"],
   ["return_scalar", "$1"]      // <-- Return annotation for second branch
   ```

2. **Stage 1 (extract_annotations)**: 
   - Extracts ALL return annotations from the token stream
   - Stores them in a HashMap keyed by rule name
   - **PROBLEM**: Only the last return annotation is kept!
   - Removes return annotation tokens from the stream

3. **Stage 2 (group_by_or)**: 
   - Splits alternatives by `|` operators
   - But return annotations are already gone!

## Proposed Solution

### Option 1: Keep Annotations in Token Stream (Recommended)

1. **Modify Stage 1**:
   - DO NOT extract return annotations
   - Keep them in the token stream
   - Only extract semantic and logging annotations

2. **Modify Stage 2**:
   - When splitting by `|`, keep return annotations with their branches
   - Each alternative gets its own token sequence including return annotation

3. **New Stage 2.5 or 3**: Extract Branch Return Annotations
   - After alternatives are separated
   - Extract return annotation from each alternative
   - Store in a new structure: `HashMap<String, Vec<Option<ReturnAnnotation>>>`
   - Index corresponds to alternative index

### Option 2: Extract with Position Info

1. **Modify Stage 1**:
   - Track position of each return annotation in the token stream
   - Store with branch index information

2. **Create mapping structure**:
   ```rust
   pub struct BranchReturnAnnotation {
       pub rule_name: String,
       pub branch_index: usize,
       pub annotation: ReturnAnnotation,
   }
   ```

## Implementation Steps

### Step 1: Modify AST Pipeline Data Structures

```rust
// In ast_pipeline.rs
pub struct Annotations {
    pub semantic_annotations: HashMap<String, Vec<String>>,
    pub logging_annotations: HashMap<String, Vec<String>>,
    // REMOVE: pub return_annotations: HashMap<String, ReturnAnnotation>,
    // ADD: Branch-level return annotations
    pub branch_return_annotations: HashMap<String, Vec<Option<ReturnAnnotation>>>,
}
```

### Step 2: Modify extract_annotations

Don't extract return annotations in Stage 1, leave them in the token stream.

### Step 3: Add return annotation extraction after grouping

After Stage 2 (group_by_or), extract return annotations from each alternative.

### Step 4: Modify Code Generation

Update the code generator to use branch-specific return annotations.

## Code Generation Changes

### Current (Wrong):
```rust
// In generate_sequence_code
let return_annotation = annotations.return_annotations.get(rule_name);
```

### New (Correct):
```rust
// In generate_n_branch_template
for (branch_idx, alt) in alternatives.iter().enumerate() {
    let branch_annotation = annotations
        .branch_return_annotations
        .get(rule_name)
        .and_then(|branches| branches.get(branch_idx))
        .and_then(|opt| opt.as_ref());
    
    // Generate code with branch-specific annotation
    if let Some(annotation) = branch_annotation {
        // Apply return annotation for this branch
    }
}
```

## Bootstrap Mode Considerations

For bootstrap mode (semantic_annotation_parser, return_annotation_parser):
- Use `ReturnAnnotationHandler` with bootstrap mode
- Limited to scalar, array, object patterns
- No dot notation, array slicing, etc.

## Testing Strategy

1. Create test EBNF with multiple alternatives having different return annotations
2. Verify each branch produces correct AST based on its annotation
3. Test bootstrap mode limitations

## Example EBNF

```ebnf
value := object_def -> {type: "object", data: $1}
      | array_def -> {type: "array", data: $1}  
      | simple_value -> {type: "simple", data: $1}
```

Each alternative has its own return annotation that should be applied when that branch matches.