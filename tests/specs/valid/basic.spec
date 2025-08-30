# TEST_MODE: full_pipeline
# EXPECT: pass
Lispish::
 -> parenthesis     {return call(parenthesis)}
 -> parenthesis[1]  {say "(Lispish) -E- Syntax Error"; exit 1}
 -> comments

parenthesis: /\(/ /\)/
I {
 my @submatchs; 
 my @word;
 my $retv
}

 -> parenthesis       {
 push @submatchs, join("", @word) if @word;
 @word = ();
 $retv = call(parenthesis)
}

 -> spaces            {$retv = call(spaces)}
 -> dquotes           {$retv = call(dquotes)}
 -> others            {$retv = call(others)}
 -> comments          {$retv = call(comments)}

 -> parenthesis[1]    {
 push @submatchs, join("", @word) if @word;
 return @submatchs >= 1 ? [$submatchs[0], @submatchs == 1 ? undef : [@submatchs[1 .. $#submatchs]]] : [undef]
}

 LE {
  my $is_ref = ref($retv);
  unless ($is_ref && $is_ref eq 'HASH') {
   push @submatchs, $retv;
  } elsif ($retv->{type} eq 'SPACE') {
    push @submatchs, join("", @word) if @word;
    @word = ()
  } elsif ($retv->{type} ne 'COMMENTS') {
    push @word, $retv->{content}
  } 
 }

dquotes: /"(.*?)(?<!\\)"/     I {
 return {type=>'DQUOTES', content=>$IMATCH_LIST[0]}
}

spaces: /\s+/               I {
 return {type=>'SPACE',    content=>$IMATCH}
}

others: /[^\s"\{\}\(\)\[\];]+/  I {
 return {type=>'OTHERS',   content=>$IMATCH}
}
                              
comments: /;.*\n/           I {
 return {type=>'COMMENTS', content=>$IMATCH}
} 