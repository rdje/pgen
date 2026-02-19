# Grammar parser for := style rules
# This spec will parse EBNF-style grammar definitions

grammar_file:: I {
  my @rules;
  my @rule;
  my $rule;
  my @includes;
  my @semantic_annotations;
  my $on;
}

LX {
  if ($rule) {
    push @rules, [$rule, @rule];
  }

  return [@includes, @rules]
}

-> include_dir.push(includes)
-> include_file.push(includes)

-> grammar_rule   {
  if ($rule) {
    push @rules, [$rule, @rule];
  }

  @rule=(@semantic_annotations);
  @semantic_annotations=();

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

-> semantic_annotation.push(semantic_annotations)
-> logging_annotation  {
  if ($on) { 
    push @rule, call(logging_annotation)
  } else {
    say "Error: Logging annotation occurrence with no container rule context";
    return undef
  }
}

-> whitespace
-> comment

grammar_rule: /(?m)^\s*[[:alpha:]_]\w*\s*:{,2}=/  I {return ["rule", ($IMATCH =~ /([[:alpha:]_]\w*)/o)[0]]}
rule_name: /\b[[:alpha:]_]\w*/                    I {return ["rule_reference", $IMATCH]}

quoted_string: /"[^"]*"|'[^']*'/    I {$IMATCH =~ s/^(?:'|")|(?:'|")$//g; return ["quoted_string", $IMATCH]}
number: /\b\d+\b/                   I {return ["number", $IMATCH]}
quantifier: /\{\s*(?:\d+(?:\s*,\s*\d*)?|,\s*\d+)\s*\}/  I {$IMATCH =~ s/\{|\}//go; return ["quantifier", $IMATCH]}
pipe_operator: /\|/                 I {return ["operator", $IMATCH]}
plus_operator: /\+/                 I {return ["operator", $IMATCH]}
star_operator: /\*/                 I {return ["operator", $IMATCH]}
question_operator: /\?/             I {return ["operator", $IMATCH]}
return_scalar: /->\s*\K(?:\$\d+|"[^"]*"|'[^']*')/       I {return ["return_scalar", $IMATCH]}
return_array: /->\s*\K(?&array_structure)(?(DEFINE)(?<array_structure>\[(?&content)\])(?<object_structure>\{(?&content)\})(?<content>(?:[^{}\[\]]*|(?&array_structure)|(?&object_structure))*))/     I {return ["return_array", $IMATCH]}
return_object: /->\s*\K(?&object_structure)(?(DEFINE)(?<array_structure>\[(?&content)\])(?<object_structure>\{(?&content)\})(?<content>(?:[^{}\[\]]*|(?&array_structure)|(?&object_structure))*))/   I {return ["return_object", $IMATCH]}
open_paren: /\(/                    I {return ["group_open", $IMATCH]}
close_paren: /\)/                   I {return ["group_close", $IMATCH]}
probability: /@\d+%?/               I {$IMATCH =~ s/@|%//g; return ["probability", $IMATCH]}
regex: /(?<!\\)\/.+?(?<!\\)\//      I {$IMATCH =~ s/^\/|\/$//g; return ["regex", $IMATCH]}
whitespace: /\s+/
comment: /#.*/
include_dir: /\b(?:include_)?dir\(\s*[^)]*?\s*\)/ I {
  my $args = $IMATCH;
  $args =~ s/^\s*(?:include_)?dir\(\s*//;
  $args =~ s/\s*\)\s*$//;
  my @parts = grep { length($_) } map { my $v = $_; $v =~ s/^\s+|\s+$//g; $v } split /\s*,\s*/, $args;
  return ["include_dir", \@parts]
}
include_file: /\b(?:include(?:_file)?|file)\(\s*[^)]*?\s*\)/ I {
  my $args = $IMATCH;
  $args =~ s/^\s*(?:include(?:_file)?|file)\(\s*//;
  $args =~ s/\s*\)\s*$//;
  my @parts = grep { length($_) } map { my $v = $_; $v =~ s/^\s+|\s+$//g; $v } split /\s*,\s*/, $args;
  return ["include_file", \@parts]
}

semantic_annotation: /@(\w+)\s*:\s*/
-> semantic_annotation 	{BACKTRACK(); my $c = $CAPTURE; $c =~ s/\s*$//o; $c =~ s/^"|"$//go; return ['semantic_annotation', [$IMATCH_LIST[0], $c]]}
-> grammar_rule		{BACKTRACK(); my $c = $CAPTURE; $c =~ s/\s*$//o; $c =~ s/^"|"$//go; return ['semantic_annotation', [$IMATCH_LIST[0], $c]]}

logging_annotation: /@((?:log|debug|trace|benchmark|profile|timing)_\w+)\s*\(\s*/ /\s*\)/ 	@move_pos
I {$IMATCH =~ s/@|\s*\(//go}

-> quoted_string {
  push @logging_annotation, call(quoted_string)->[1]
}
-> comma.capture_if
-> logging_annotation[1] {
  CAPTURE_IF();
  return ['logging_annotation', [$IMATCH, [@logging_annotation]]]
}

comma: /\s*,\s*/
