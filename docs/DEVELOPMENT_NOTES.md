# DEVELOPMENT_NOTES.md - Technical Knowledge Base

## Architecture Insights

### Universal Test Infrastructure (2024-12)
- **Key Decision**: Moved from multiple parser-specific test runners to ONE universal test runner
- **Benefits**: No code duplication, zero maintenance overhead for new parsers
- **Format**: JSON-based test data with parser field, input, expected output, and tags
- **Location**: All test data under `rust/test_data/` organized by parser

### Return Annotations Implementation
- **Syntax**: `-> {positional}` for positional references, `-> {::extraction}` for extraction operators
- **AST Integration**: Unified AST with ReturnAnnotation enum handling both types
- **Parser Support**: Bootstrap, Unified, External parsers all support return annotations
- **Test Coverage**: Comprehensive JSON test suites under `rust/test_data/return_annotations/`

### Parser Architecture
- **Bootstrap Parser**: Initial parser for basic pgen grammar
- **Unified Parser**: Advanced parser with full feature support
- **External Parser**: Parser for third-party grammars
- **Stub Parser**: Testing-only parser for isolated feature testing

## Best Practices Discovered

### Testing Philosophy
1. **Tests as Data**: All tests should be JSON data files, not code
2. **Never Throw Away Tests**: Even experimental tests should be kept for regression
3. **Group by Feature**: Test organization should follow feature boundaries
4. **Tag Everything**: Use tags for flexible test filtering and categorization

### Parser Development
1. **Start with AST**: Define the AST structure before implementing parsing
2. **Test Data First**: Create test data before implementing features
3. **Stub When Needed**: Use stub parsers to test features in isolation
4. **Unified Format**: All parsers should produce compatible AST structures

### Code Organization
1. **Feature Modules**: Group related functionality in feature-specific modules
2. **Clear Boundaries**: Keep parser logic separate from AST manipulation
3. **Consistent Naming**: Use consistent naming across parsers and tests

## Complex Systems Understanding

### FSM::CoreAST Integration
- **Purpose**: Core abstract syntax tree for FSM generation
- **Location**: `/Users/richarddje/Downloads/AFX/fsm/afx/cursor/fx/perl/FSM/`
- **Key Insight**: FSM generation requires careful handling of state transitions and actions
- **Debug Strategy**: Extensive logging with file/function context for traceability

### Test Runner Implementation
- **Key Components**:
  - `TestSuite`: Container for related tests
  - `TestCase`: Individual test with input/expected output
  - `UniversalTestRunner`: Core runner that delegates to parser-specific implementations
- **Extension Points**: New parsers just need to implement parser function registration

## Technical Debt and Future Enhancements

### Near-term Improvements
1. **CI Integration**: Integrate JSON test runner into CI pipeline
2. **Performance Metrics**: Add timing and memory usage to test results
3. **Parallel Execution**: Run test suites in parallel for faster feedback
4. **Coverage Reporting**: Track which grammar rules are covered by tests

### Long-term Vision
1. **Visual Test Explorer**: Web-based UI for browsing and running tests
2. **Test Generation**: Automatically generate tests from grammar changes
3. **Fuzzing Support**: Use test infrastructure for grammar fuzzing
4. **Cross-Language Testing**: Share JSON tests with implementations in other languages

## Design Principles

### Simplicity Over Complexity
- One test runner is better than many
- JSON is better than code for test data
- Consistent formats reduce cognitive load

### Extensibility Without Breaking
- New features should not break existing tests
- Parser additions should not require runner changes
- Test format extensions should be backward compatible

### Developer Experience
- Running tests should be trivial
- Adding tests should require no code
- Debugging failures should be straightforward

## Known Issues and Workarounds

### Parser Generation Cycles
- **Issue**: Some parsers depend on others being generated first
- **Workaround**: Use stub parsers for testing features in isolation
- **Long-term Fix**: Implement proper dependency resolution in build system

### Test Data Validation
- **Issue**: JSON schema not enforced, malformed tests fail at runtime
- **Workaround**: Manual validation during test creation
- **Long-term Fix**: Add JSON schema validation to test runner

## Performance Considerations

### Test Execution
- JSON parsing overhead is negligible compared to parser execution
- File I/O is cached by OS for repeated test runs
- Parallel execution would provide biggest performance gain

### Memory Usage
- Test data is loaded on demand, not all at once
- Parser state is reset between tests to avoid memory leaks
- Large test suites might benefit from streaming JSON parsing

## Integration Points

### Git Workflow
- Tests are version controlled alongside code
- PR reviews should include test additions
- Breaking changes require test updates

### Documentation
- TEST_INFRASTRUCTURE.md is the source of truth
- Each parser should document its test organization
- Complex features should have test documentation

## Lessons Learned

1. **Universal > Specific**: A universal solution eliminates maintenance burden
2. **Data > Code**: Test data is easier to maintain than test code
3. **Organization Matters**: Well-organized tests are easier to understand and maintain
4. **Documentation is Critical**: Future developers/AI need context to be productive
5. **Automation First**: Manual processes don't scale, automate everything possible

---

*Last Updated: December 2024*
*Next Update: When significant architectural changes or insights occur*