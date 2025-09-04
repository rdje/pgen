# Annotation Parser Test Suite

## Test Structure

### 1. Return Annotation Parser Tests (`return_annotation_tests.rs`)
- **Foundation layer**: Tests the generated return annotation parser directly
- **No dependencies**: Tests the raw parser functionality
- **Test cases**: Various return annotation syntax patterns

### 2. Semantic Annotation Parser Tests (`semantic_annotation_tests.rs`)  
- **Depends on**: Return annotation parser being functional
- **Tests**: Semantic annotations that contain return annotation syntax
- **Integration**: Verifies semantic parser can parse return annotations as values

### 3. Regex Parser Tests (`regex_parser_tests.rs`)
- **Depends on**: Semantic annotation parser being functional
- **Tests**: Regex parsing guided by semantic annotations
- **Integration**: Full chain functionality

### 4. Integration Tests (`integration_tests.rs`)
- **Full chain**: All parsers working together
- **End-to-end**: Real grammar files with all annotation types
- **Pipeline**: AST pipeline processing all annotation types

## Test Data Organization

```
tests/
├── annotation_parsers/
│   ├── lib.rs                           # Test utilities
│   ├── return_annotation_tests.rs       # Foundation tests
│   ├── semantic_annotation_tests.rs     # Layer 2 tests  
│   ├── regex_parser_tests.rs           # Layer 3 tests
│   ├── integration_tests.rs            # Full integration
│   └── test_data/
│       ├── return_annotations/          # Test cases for return annotations
│       ├── semantic_annotations/        # Test cases for semantic annotations
│       ├── regex_patterns/              # Test cases for regex parsing
│       └── integration/                 # Full grammar files
```

## Dependency Testing Strategy

1. **Bottom-up**: Test foundation first, then build up
2. **Isolated**: Each layer can be tested independently  
3. **Progressive**: Each layer assumes the layer below works
4. **Comprehensive**: Full integration tests at the top
