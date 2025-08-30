# EBNF Parser Generator - Quick Reference

## Syntax Reference

### Basic EBNF Constructs

| Syntax | Meaning | Example |
|--------|---------|---------|
| `rule := definition` | Rule definition | `number := digit+` |
| `|` | Alternative (OR) | `op := '+' \| '-'` |
| `+` | One or more | `digits := digit+` |
| `*` | Zero or more | `spaces := ' '*` |
| `?` | Optional (zero or one) | `sign := ('+'|'-')?` |
| `{n,m}` | Between n and m | `id := letter{1,10}` |
| `"text"` | Terminal string | `keyword := "if"` |
| `'text'` | Terminal string | `op := '+'` |
| `()` | Grouping | `expr := ('+'\|'-') term` |

### Probability Annotations

| Syntax | Meaning | Example |
|--------|---------|---------|
| `@n%` | Probability weight | `op := '+' @60% \| '-' @40%` |
| Auto-normalize | Unspecified get remainder | `op := '+' @30% \| '-' \| '*'` → `-` gets 35%, `*` gets 35% |

### Comments

```ebnf
# This is a comment
rule := definition  # End-of-line comment
```

## Command Line Tools

### Generate Parser

```bash
# Basic parser generation
perl backtracking_parser_generator.pl grammar.ebnf > parser.pm

# With debug info
perl backtracking_parser_generator.pl grammar.ebnf debug > parser.pm
```

### Generate Test Inputs

```bash
# Basic generation
perl ebnf_input_generator.pl grammar.ebnf

# With options
perl ebnf_input_generator.pl grammar.ebnf \
    --count 50 \
    --max-depth 10 \
    --seed 12345
```

### Test Generated Parser

```perl
# Load and test parser
require "./parser.pm";
my $input = "test string";
my $result = yapg::BacktrackingParser::parse(\$input);
print defined($result) ? "PASS" : "FAIL";
```

## Common Grammar Patterns

### Numbers and Identifiers

```ebnf
# Integer numbers
number := digit+
digit := "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"

# Floating point
float := number ('.' number)?

# Identifiers  
identifier := letter (letter | digit)*
letter := "a" | "b" | ... | "z" | "A" | ... | "Z"
```

### Expressions with Precedence

```ebnf
# Arithmetic expressions
expression := term (('+' | '-') term)*
term := factor (('*' | '/') factor)*  
factor := '(' expression ')' | number | identifier
```

### Lists and Sequences

```ebnf
# Comma-separated list
list := item (',' item)*
item := identifier | number

# Optional trailing comma
list := item (',' item)* ','?

# Empty list allowed
list := (item (',' item)*)?
```

### Realistic Probability Distributions

```ebnf
# Common vs rare constructs
statement := assignment @60% | if_stmt @20% | loop @15% | function_call @5%

# Realistic identifier patterns  
identifier := common_word @70% | rare_word @20% | generated_id @10%
common_word := "var" | "temp" | "data" | "result" | "value"
rare_word := "configuration" | "implementation" | "parameter"
generated_id := letter+ digit*

# Natural number distribution (small numbers more common)
number := small @70% | medium @25% | large @5%
small := "1" | "2" | "3" | "4" | "5"
medium := "10" | "20" | "50" | "100"  
large := "1000" | "5000" | "10000"
```

## File Structure Examples

### Simple Grammar File

```ebnf
# simple.ebnf
expression := number '+' number @60% | number @40%
number := "1" @30% | "2" @25% | "3" @25% | "4" @20%
```

### Complex Grammar File

```ebnf
# json.ebnf  
json := object | array | string | number | boolean | null

object := '{' (pair (',' pair)*)? '}'
pair := string ':' json

array := '[' (json (',' json)*)? ']'

string := '"' char* '"'
char := letter | digit | ' ' | '_'

number := digit+ ('.' digit+)?
boolean := "true" @50% | "false" @50%
null := "null"

letter := "a" | "b" | "c" | ... # (simplified)
digit := "0" | "1" | "2" | ... # (simplified)
```

## Workflow Examples

### Complete Workflow

```bash
# 1. Create grammar
cat > my_grammar.ebnf << 'EOF'
expr := term '+' term @50% | term '-' term @30% | term @20%
term := number | identifier
number := digit+
identifier := letter+
digit := "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
letter := "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j"
EOF

# 2. Generate parser
perl backtracking_parser_generator.pl my_grammar.ebnf > my_parser.pm

# 3. Generate test inputs
perl ebnf_input_generator.pl my_grammar.ebnf --count 20 > test_inputs.txt

# 4. Test parser
perl -e '
require "./my_parser.pm";
open my $fh, "<", "test_inputs.txt";
while (my $line = <$fh>) {
    chomp $line;
    next if $line =~ /^#/ || $line =~ /^\s*$/;
    my $result = yapg::BacktrackingParser::parse(\$line);
    printf "%-15s: %s\n", "[$line]", (defined($result) ? "PASS" : "FAIL");
}
'
```

### Debugging Failed Parsing

```perl
# Debug individual rules
require "./my_parser.pm";
my $input = "failed_input";

print "Testing top-level rule...\n";
my $result = yapg::BacktrackingParser::parse_expr(\$input, 0);
print defined($result) ? "PASS" : "FAIL";

print "Testing sub-rules...\n";  
my $term_result = yapg::BacktrackingParser::parse_term(\$input, 0);
my $number_result = yapg::BacktrackingParser::parse_number(\$input, 0);
```

## Error Messages and Solutions

### Common Errors

**Error**: `ERROR: Probabilities sum to 120%, must equal 100%`
```ebnf
# Problem
expr := "a" @50% | "b" @40% | "c" @30%  # 50+40+30 = 120%

# Solution  
expr := "a" @50% | "b" @30% | "c" @20%  # 50+30+20 = 100%
```

**Error**: `Undefined rule referenced: 'unknown_rule'`
```ebnf
# Problem
expr := unknown_rule '+' term

# Solution: Define the missing rule
expr := number '+' term
number := digit+
```

**Error**: `Can't locate parser.pm`
```bash
# Problem: Wrong path or parser not generated

# Solution: Check file exists and use correct path
ls -la *.pm
perl -I. -e 'require "./parser.pm"'
```

### Performance Issues

**Slow parsing**: Check for left recursion
```ebnf
# Problem (infinite recursion)
expr := expr '+' term | term

# Solution (right recursion)  
expr := term ('+' term)*
```

**Memory usage**: Limit recursion depth
```bash
# Problem: Deep recursion
perl ebnf_input_generator.pl grammar.ebnf --max-depth 20

# Solution: Reduce depth
perl ebnf_input_generator.pl grammar.ebnf --max-depth 5
```

## Best Practices

### Grammar Design

1. **Start Simple**: Begin with basic rules, add complexity gradually
2. **Test Early**: Generate parser and inputs frequently during development
3. **Avoid Left Recursion**: Use right recursion or iteration patterns
4. **Realistic Probabilities**: Base percentages on real-world data
5. **Meaningful Names**: Use descriptive rule names

### Probability Design

1. **Always Sum to 100%**: Explicitly specify or let auto-normalization handle it
2. **Reflect Reality**: Common constructs should have higher probabilities
3. **Test Distribution**: Generate large samples to verify probability adherence
4. **Document Assumptions**: Comment why certain probabilities were chosen

### Testing Strategy

1. **Unit Test Rules**: Test individual rules before combining
2. **Edge Cases**: Test empty inputs, maximum lengths, boundary conditions
3. **Integration Test**: Full workflow from grammar to parsed inputs
4. **Performance Test**: Large inputs and complex grammars

### Debugging Tips

1. **Start Small**: Debug with minimal examples
2. **Check Probabilities**: Ensure they sum to 100%
3. **Verify Rule Names**: Confirm all referenced rules are defined
4. **Test Incrementally**: Add rules one at a time

---

## Quick Troubleshooting

| Problem | Quick Check | Solution |
|---------|-------------|----------|
| Parser fails | Rule defined? | Add missing rule definition |
| Probability error | Sum = 100%? | Fix probability values |
| Generation fails | Left recursion? | Redesign recursive rules |
| Slow performance | Deep recursion? | Reduce max-depth |
| No matches | Correct syntax? | Check EBNF syntax |

For detailed documentation, see `EBNF_PARSER_GENERATOR.md`  
For improvement plans, see `EBNF_IMPROVEMENT_ROADMAP.md`





