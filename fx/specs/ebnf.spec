# Grammar parser for := style rules
# This spec will parse EBNF-style grammar definitions

grammar_file:: I {
  my @rules;
  my @rule;
  my $rule;
  my $on;
}

LX {
  if ($rule) {
    push @rules, [$rule, @rule];
  }

  return [@rules]
}

-> grammar_rule   {
  if ($rule) {
    push @rules, [$rule, @rule];
    @rule=()
  }

  $rule = call(grammar_rule);
  $on=1
}

-> rule_name  {
  if ($on) {
    push @rule, call(rule_name)
  } else {
    say "Error: Rule name '$LMATCH' reference with no container rule context";
    return undef
  }
}

-> quoted_string   {
  if ($on) {
    push @rule, call(quoted_string)
  } else {
    say "Error: Quoted string <$LMATCH> occurrence with no container rule context";
    return undef
  }
}

-> number  {
  if ($on) { 
    push @rule, call(number)
  } else {
    say "Error: Number '$LMATCH' occurrence with no container rule context";
    return undef
  }
}

-> quantifier  {
  if ($on) { 
    push @rule, call(quantifier)
  } else {
    say "Error: Quantifier occurrence with no container rule context";
    return undef
  }
}

-> plus_operator  {
  if ($on) { 
    push @rule, call(plus_operator)
  } else {
    say "Error: '+' operator occurrence with no container rule context";
    return undef
  }
}

-> return_scalar  {
  if ($on) { 
    push @rule, call(return_scalar)
  } else {
    say "Error: Scalar return annotation occurrence with no container rule context";
    return undef
  }
}

-> return_array  {
  if ($on) { 
    push @rule, call(return_array)
  } else {
    say "Error: Array return annotation occurrence with no container rule context";
    return undef
  }
}

-> return_object  {
  if ($on) { 
    push @rule, call(return_object)
  } else {
    say "Error: Object return annotation occurrence with no container rule context";
    return undef
  }
}

-> star_operator  {
  if ($on) { 
    push @rule, call(star_operator)
  } else {
    say "Error: '*' operator occurrence with no container rule context";
    return undef
  }
}

-> question_operator {
  if ($on) { 
    push @rule, call(question_operator)
  } else {
    say "Error: '?' operator occurrence with no container rule context";
    return undef
  }
}

-> pipe_operator  {
  if ($on) { 
    push @rule, call(pipe_operator)
  } else {
    say "Error: '|' operator occurrence with no container rule context";
    return undef
  }
}

-> open_paren   {
  if ($on) { 
    push @rule, call(open_paren)
  } else {
    say "Error: '(' occurrence with no container rule context";
    return undef
  }
}

-> close_paren  {
  if ($on) { 
    push @rule, call(close_paren)
  } else {
    say "Error: ')' occurrence with no container rule context";
    return undef
  }
}

-> probability   {
  if ($on) { 
    push @rule, call(probability)
  } else {
    say "Error: Probability occurrence with no container rule context";
    return undef
  }
}

-> regex   {
  if ($on) { 
    push @rule, call(regex)
  } else {
    say "Error: Regex occurrence with no container rule context";
    return undef
  }
}

-> whitespace
-> comment

grammar_rule: /[[:alpha:]_]\w*\s*:=/  I {return ["rule", ($IMATCH =~ /([[:alpha:]_]\w*)/o)[0]]}
rule_name: /[[:alpha:]_]\w*/        I {return ["rule_reference", $IMATCH]}

quoted_string: /"[^"]*"|'[^']*'/    I {$IMATCH =~ s/'|"//g; return ["quoted_string", $IMATCH]}
number: /\d+/                       I {return ["number", $IMATCH]}
quantifier: /\{\s*(?:\d+(?:\s*,\s*\d*)?|,\s*\d+)\s*\}/  I {$IMATCH =~ s/\{|\}//go; return ["quantifier", $IMATCH]}
pipe_operator: /\|/                 I {return ["operator", $IMATCH]}
plus_operator: /\+/                 I {return ["operator", $IMATCH]}
star_operator: /\*/                 I {return ["operator", $IMATCH]}
question_operator: /\?/             I {return ["operator", $IMATCH]}
return_scalar: /->\s*\K(?:\$\d+|"[^"]*"|'[^']*')/       I {return ["return_scalar", $IMATCH]}
return_array: /->\s*\K(?&array_structure)(?(DEFINE)(?<array_structure>\[(?&content)\])(?<object_structure>\{(?&content)\})(?<content>(?:[^{}\[\]]*|(?&array_structure)|(?&object_structure))*))/   I {return ["return_array", $IMATCH]}
return_object: /->\s*\K(?&object_structure)(?(DEFINE)(?<array_structure>\[(?&content)\])(?<object_structure>\{(?&content)\})(?<content>(?:[^{}\[\]]*|(?&array_structure)|(?&object_structure))*))/   I {return ["return_object", $IMATCH]}
open_paren: /\(/                    I {return ["group_open", $IMATCH]}
close_paren: /\)/                   I {return ["group_close", $IMATCH]}
probability: /@\d+%/                I {$IMATCH =~ s/@|%//g; return ["probability", $IMATCH]}
regex: /(?<!\\)\/.+?(?<!\\)\//      I {$IMATCH =~ s/^\/|\/$//g; return ["regex", $IMATCH]}
whitespace: /\s+/
comment: /#.*/
