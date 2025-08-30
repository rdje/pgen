pplugin_top::   I {my @defs; my $retv}
 -> comment       {next}
 -> subdef        {$retv = call(subdef)}

LE {return undef unless defined $retv; push @defs, @$retv}
LX {return {@defs}}


subdef: /(?<subname>\w\S*)\s*(?<!\\)\{/ /(?<!\\)\}/
# -> comment
 -> curlyb
 -> dquotes
 -> squotes
 -> subdef[1]	{return [$IMATCH_HASH{subname}, sub {eval substr($$STRING, $IPOS, $LSPOS - $IPOS -1)}]}


curlyb: /(?<!\\)\{/ /(?<!\\)\}/
# -> comment
 -> curlyb
 -> dquotes
 -> squotes
 -> curlyb[1]	{return}


comment: /#.*/
dquotes: /(?<!\\)".*?(?<!\\)"/
squotes: /(?<!\\)'.*?(?<!\\)'/

