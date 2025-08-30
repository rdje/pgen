top::            I {my @blocks}

 -> comments
 -> begin_end_blocks          {
	                       my $retv = call(begin_end_blocks);
	                       push @blocks, $retv if $retv
		              }

 LX {return @blocks ? \@blocks : undef}


begin_end_blocks: /\bBEGIN\s+\w+/ /\bEND\s+\w+/  I {my ($block_namei) = $IMATCH =~ /(\w+)$/; print "begin_end_blocks: BEGIN   ($IMATCH)\n"; my $retv; my @assigns; my @keyval_pairs}

 -> comments
 -> anyvariable                       {
	                               if (@keyval_pairs) {
                                        push @assigns, [@keyval_pairs];
					@keyval_pairs = (call(anyvariable));
				       }
                                      }

 -> multiline_value                   {push @keyval_pairs, call(multiline_value)}
 -> singleline_value                  {push @keyval_pairs, call(singleline_value)}
 -> begin_end_blocks[1]               {
	                               my ($block_namee) = $LMATCH =~ /(\w+)$/;

				       do {
                                         my @startline = substr($$STRING, 0, $IPOS)  =~ /\n/g;  
                                         my @endline   = substr($$STRING, 0, $LSPOS-length($LMATCH)) =~ /\n/g;  
                                         print "(simenv) -E- BEGIN Block Name '$block_namei' and END Block name '$block_namee' do not match.\n"; 
                                         print "             BEGIN statement is on line ".(@startline +1)." while END statement is on line ".(@endline +1)."\n"; 
					 exit
				        } unless $block_namee eq $block_namei;

	                               push @assigns, [@keyval_pairs] if @keyval_pairs;
	                               print "begin_end_blocks: END    ($LMATCH)\n";
				       return @assigns ? {name=>$block_namei, content=>\@assigns} : undef
			              }

 LX {my @startline = substr($$STRING, 0, $IPOS) =~ /\n/g;  
     print "(simenv) -E- END Block statement not found for *begin_end_blocks* starting on line ".(@startline +1)."\n"; 
     exit}

      
anyvariable: /\S+\s*(?==)/ I {$IMATCH =~ /(\S+)/; print "anyvariable: VARIABLE NAME ($1)\n"; return {type=>'anyvariable', content=>$1}}

multiline_value: /=\s*\{/    /\}/ I {print "multiline_value: START\n"}
 -> curlybrace
 -> multiline_value[1]	     {
	                      print "multiline_value: CLOSING curly brace\n"; 
			      print "<".substr($$STRING, $IPOS, $LSPOS - $IPOS -1).">\n"; 
			      return {type=>'multiline_value', content=>substr($$STRING, $IPOS, $LSPOS - $IPOS -1)}
		             }

 LX {my @startline = substr($$STRING, 0, $IPOS) =~ /\n/g; 
     print "(simenv) -E- Closing parenthesis not found for *multiline_value* starting on line ".(@startline +1)."\n"; 
     exit}


singleline_value:    /=/ /(?<!\\)\n|\b(?=END\s+\w+)/ I {my $last_pos=$IPOS; my @matches} 
 -> perl_command_substitution          {push @matches, call(perl_command_substitution);   $last_pos = pos($$STRING)}
 -> command_substitution               {push @matches, call(command_substitution);        $last_pos = pos($$STRING)}
 -> bvariable_substitution             {push @matches, call(bvariable_substitution);      $last_pos = pos($$STRING)}
 -> variable_substitution              {push @matches, call(variable_substitution);       $last_pos = pos($$STRING)}
 -> squotes                            {push @matches, call(squotes);                     $last_pos = pos($$STRING)}
 -> dquotes                            {push @matches, call(dquotes);                     $last_pos = pos($$STRING)}
 -> perl_squotes                       {push @matches, call(perl_squotes);                $last_pos = pos($$STRING)}
 -> perl_dquotes                       {push @matches, call(perl_dquotes);                $last_pos = pos($$STRING)}
 -> bs_nl                              {push @matches, call(bs_nl);                       $last_pos = pos($$STRING)}
 -> singleline_value[1]     {print "singleline_value: END\n";  print "<".substr($$STRING, $IPOS, $LSPOS - $IPOS -1).">\n";
	 print "singleline_value:<<$_>>\n" foreach (@matches);
	 return {type=>'singleline_value', content=> @matches ? \@matches : undef}
   }

 LS {my $shift = $LSPOS - $last_pos - length($LMATCH); push @matches, {type=>'verbatim', content=>substr($$STRING, $last_pos, $shift)} if $shift}
 LX {my @startline = substr($$STRING, 0, $IPOS) =~ /\n/g;  
     print "(simenv) -E- End of Line not found for *singleline_value* starting on line ".(@startline +1)."\n"; 
     exit}


bs_nl: /\\\n\s*/                            I {print "bs_nl: SEEN\n"; return "**BS_NL**"}
squotes: /'/ /(?<!\\)'/                     I {print "squotes: START\n"}
 -> squotes[1]                                {
	                                       print "squotes: END\n";  
					       print "<".substr($$STRING, $IPOS, $LSPOS - $IPOS -1).">\n"; 
					       return {type=>'squotes', content=>substr($$STRING, $IPOS, $LSPOS - $IPOS -1)}
				              }

 LX {my @startline = substr($$STRING, 0, $IPOS) =~ /\n/g;  
     print "(simenv) -E- Closing tick not found for *$squotes* starting on line ".(@startline +1)."\n"; 
     exit}

dquotes: /"/ /(?<!\\)"/                     I {print "dquotes: START\n"; my @matches; my $last_pos=$IPOS}
 -> bvariable_substitution                    {push @matches, call(bvariable_substitution);   $last_pos = pos($$STRING)}
 -> variable_substitution                     {push @matches, call(bvariable_substitution);   $last_pos = pos($$STRING)}
 -> dquotes[1]                                {print "dquotes: END\n";  print "<".substr($$STRING, $IPOS, $LSPOS - $IPOS -1).">\n"; 
	 print "perl_dquotes:<<$_>>\n" foreach (@matches);
         return {type=>'dquotes', content=> @matches ? \@matches : undef}}

 LS {my $shift = $LSPOS - $last_pos - length($LMATCH); push @matches, substr($$STRING, $last_pos, $shift) if $shift}
 LX {my @startline = substr($$STRING, 0, $IPOS) =~ /\n/g;  
     print "(simenv) -E- Closing parenthesis not found for *dquotes* starting on line ".(@startline +1)."\n"; 
     exit}


perl_squotes: /q\(/  /\)/                   I {print "perl_squotes: START\n"}
 -> parenthesis
 -> perl_squotes[1]                           {
	                                       print "perl_squotes: END\n"; 
					       print "<".substr($$STRING, $IPOS, $LSPOS - $IPOS -1).">\n"; 
					       return {type=>'squotes', content=>substr($$STRING, $IPOS, $LSPOS - $IPOS -1)}
				              }

 LX {my @startline = substr($$STRING, 0, $IPOS) =~ /\n/g;  
     print "(simenv) -E- Closing Parenthesis not found for *$perl_squotes* starting on line ".(@startline +1)."\n"; 
     exit}

perl_dquotes: /qq\(/  /\)/                  I {print "perl_dquotes: START\n"; my @matches; my $last_pos=$IPOS}
 -> parenthesis
 -> bvariable_substitution                    {push @matches, call(bvariable_substitution);   $last_pos = pos($$STRING)}
 -> variable_substitution                     {push @matches, call(bvariable_substitution);   $last_pos = pos($$STRING)}
 -> perl_dquotes[1]                           {print "perl_dquotes: END\n"; print "<".substr($$STRING, $IPOS, $LSPOS - $IPOS -1).">\n"; 
	 print "perl_dquotes:<<$_>>\n" foreach (@matches);
         return {type=>'dquotes', content=> @matches ? \@matches : undef}}

 LS {my $shift = $LSPOS - $last_pos - length($LMATCH); push @matches, substr($$STRING, $last_pos, $shift) if $shift}
 LX {my @startline = substr($$STRING, 0, $IPOS) =~ /\n/g;  
     print "(simenv) -E- Closing parenthesis not found for *perl_dquotes* starting on line ".(@startline +1)."\n"; 
     exit}


command_substitution: /`/  /(?<!\\)`/       I {print "command_substitution: START\n" my @matches; my $last_pos=$IPOS}
 -> bvariable_substitution                    {push @matches, call(bvariable_substitution);   $last_pos = pos($$STRING)}
 -> variable_substitution                     {push @matches, call(bvariable_substitution);   $last_pos = pos($$STRING)}
 -> command_substitution[1]                   {
	                                       print "command_substitution: END\n"; print "<".substr($$STRING, $IPOS, $LSPOS - $IPOS -1).">\n"; 
	                                       print "command_substitution:<<$_>>\n" foreach (@matches);
                                               return {type=>'command_substitution', content=> @matches ? \@matches : undef}
				              }

 LS {my $shift = $LSPOS - $last_pos - length($LMATCH); push @matches, substr($$STRING, $last_pos, $shift) if $shift}
 LX {my @startline = substr($$STRING, 0, $IPOS) =~ /\n/g;  
     print "(simenv) -E- Unmatched back-tick for *command_substitution* starting on line ".(@startline +1)."\n"; 
     exit}


perl_command_substitution: /qx\(/  /\)/     I {print "perl_command_substitution: START\n"; my @matches; my $last_pos=$IPOS}
 -> bvariable_substitution		      {push @matches, call(bvariable_substitution);   $last_pos = pos($$STRING)}
 -> variable_substitution                     {push @matches, call(variable_substitution);    $last_pos = pos($$STRING)}
 -> perl_command_substitution[1]              {print "perl_command_substitution: END\n"; print "<".substr($$STRING, $IPOS, $LSPOS - $IPOS -1).">\n"; 
	 print "perl_command_substitution:<<$_>>\n" foreach (@matches);
	 return {type=>'command_substitution', content=> @matches ? \@matches : undef}}

 LS {my $shift = $LSPOS - $last_pos - length($LMATCH); push @matches, substr($$STRING, $last_pos, $shift) if $shift}
 LX {my @startline = substr($$STRING, 0, $IPOS) =~ /\n/g;  
     print "(simenv) -E- Closing parenthesis not found for *perl_command_substitution* starting on line ".(@startline +1)."\n"; 
     exit}
 

bvariable_substitution: /(?<!\\)\$\{/ /\}/  I {print "bvariable_substitution: START\n"}
 -> curlybrace
 -> bvariable_substitution[1]                 {
	                                       print "bvariable_substitution: END\n"; 
					       print "<".substr($$STRING, $IPOS, $LSPOS - $IPOS -1).">\n"; 
					       return {type=>'bvariable_substitution', content=>substr($$STRING, $IPOS, $LSPOS - $IPOS -1)}
				              }

 LX {my @startline = substr($$STRING, 0, $IPOS) =~ /\n/g;  
     print "(simenv) -E- Closing Curly Brace not found for *bvariable_substitution* starting on line ".(@startline +1)."\n"; 
     exit}
 

variable_substitution: /(?<!\\)\$\w+/       I {
	                                       $IMATCH =~ /(\w+)$/;
	                                       print "variable_substitution: ($1)\n";
					       return {type=>'variable_substitution', content=>$1}
				              }

curlybrace: /\{/   /\}/                     I {print "curlybrace: OPENING Brace\n"}
 -> curlybrace
 -> curlybrace[1]                             {print "curlybrace: CLOSING Brace\n";  print "<".substr($$STRING, $IPOS, $LSPOS - $IPOS -1).">\n"; return}

 LX {my @startline = substr($$STRING, 0, $IPOS) =~ /\n/g;  
     print "(simenv) -E- Closing Curly Brace not found for *curlybrace* starting on line ".(@startline +1)."\n"; 
     exit}


parenthesis: /\(/   /\)/                    I {print "parenthesis: OPENING Parenthesis\n"}
 -> parenthesis
 -> parenthesis[1]                            {print "parenthesis: CLOSING Parenthesis\n";  print "<".substr($$STRING, $IPOS, $LSPOS - $IPOS -1).">\n"; return}

 LX {my @startline = substr($$STRING, 0, $IPOS) =~ /\n/g;  
     print "(simenv) -E- Closing Parenthesis not found for *parenthesis* starting on line ".(@startline +1)."\n"; 
     exit}


comments: /#.*\n/                           I {chomp $IMATCH; print "comments: <$IMATCH>\n"}

