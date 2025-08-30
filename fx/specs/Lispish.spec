Lispish::
 -> parenthesis     {return call(parenthesis)}
 -> parenthesis[1]  {say "(Lispish) -E- Syntax Error"; exit 1}
 -> comments

parenthesis: /\(/ /\)/
I {
 #say "parenthesis OPENING (";
 my @submatchs; 
 my @word;
 my $retv
}

 -> parenthesis       {
 #say "Closing WORD --> submatchs (DUE to OPENING PARENTHESIS)" if @word;
 push @submatchs, join("", @word) if @word;
 @word = ();
 $retv = call(parenthesis)
}

 -> spaces            {$retv = call(spaces)}
 -> dquotes           {$retv = call(dquotes)}
 -> sbrackets         {$retv = call(sbrackets)}
 -> curlyb            {$retv = call(curlyb)}
 -> others            {$retv = call(others)}
 -> comments          {$retv = call(comments)}

 -> parenthesis[1]    {
 #say "parenthesis CLOSING )";
 #say "Closing WORD --> submatchs (DUE to CLOSING PARENTHESIS)" if @word;
 push @submatchs, join("", @word) if @word;
 return @submatchs >= 1 ? [$submatchs[0], @submatchs == 1 ? undef : [@submatchs[1 .. $#submatchs]]] : [undef]
}
 

 LE {
  my $is_ref = ref($retv);
  unless ($is_ref && $is_ref eq 'HASH') {
  # For Opening parenthesis
   push @submatchs, $retv;
  } elsif ($retv->{type} eq 'SPACE') {
    #say "Closing WORD --> submatchs (DUE to SPACE)";
    push @submatchs, join("", @word) if @word;
    @word = ()
  } elsif ($retv->{type} ne 'COMMENTS') {
    #say "Pushing '$retv' into WORD";
    # DQUOTES + CBRACE + OTHERS
    push @word, $retv->{content}
  } 
 }

sbrackets: /(\[(?:[^\[\]]++|(?R))+\])/     I {
	#say $IMATCH; 
 return {type=>'SBRACKETS', content=>$IMATCH}
}

dquotes: /"(.*?)(?<!\\)"/     I {
 #say "dquotes<$IMATCH_LIST[0]>"; 
 return {type=>'DQUOTES', content=>$IMATCH_LIST[0]}
}

squotes: /'(.*?)(?<!\\)'/     I {
 #say "squotes<$IMATCH_LIST[0]>"; 
 return {type=>'SQUOTES', content=>$IMATCH_LIST[0]}
}

curlyb: /(?<!\\)\{/ /(?<!\\)\}/
 -> curlyb
 -> dquotes
 -> squotes
 -> curlyb[1]                 {
 #say "curlyb<".substr($$STRING, $IPOS, $LSPOS - $IPOS - 1).">"; 
 return {type=>'CBRACE',   content=>substr($$STRING, $IPOS, $LSPOS - $IPOS - 1)}
}

spaces: /\s+/               I {
	#say "spaces<$IMATCH>"; 
 return {type=>'SPACE',    content=>$IMATCH}
}

others: /[^\s"\{\}\(\)\[\];]+/  I {
 #say "others<$IMATCH>"; 
 return {type=>'OTHERS',   content=>$IMATCH}
}
			      
comments: /;.*\n/           I {
 #say "comments<$IMATCH>"; 
 return {type=>'COMMENTS', content=>$IMATCH}
}
