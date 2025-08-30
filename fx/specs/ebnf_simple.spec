# Simple EBNF spec to test token structure
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
  }
}

-> quoted_string   {
  if ($on) {
    push @rule, call(quoted_string)
  }
}

-> plus_operator  {
  if ($on) { 
    push @rule, call(plus_operator)
  }
}

-> pipe_operator  {
  if ($on) { 
    push @rule, call(pipe_operator)
  }
}

-> whitespace
-> comment

grammar_rule: /[[:alpha:]_]\w*\s*:=/  I {return ($IMATCH =~ /([[:alpha:]_]\w*)/o)[0]}
rule_name: /[[:alpha:]_]\w*/        I {return $IMATCH}
quoted_string: /"[^"]*"|'[^']*'/    I {$IMATCH =~ s/'|"//g; return $IMATCH}
pipe_operator: /\|/                 I {return $IMATCH}
plus_operator: /\+/                 I {return $IMATCH}
whitespace: /\s+/
comment: /#.*/

