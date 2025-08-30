sdc_esplit::  -> get_pinport  {push @pieces, call(get_pinport)}
I    {my @pieces; $IPOS = 0}
LS   {push @pieces, substr $$STRING, $IPOS, $LSPOS - $IPOS - length $LMATCH}
LE   {$IPOS = pos $$STRING}
LX   {push @pieces, substr $$STRING, $IPOS, length ($$STRING) - $IPOS; return [@pieces]}


get_pinport: /\[\s*((?:get_port|get_pin)\w?\s+)/ /\]/
-> oc_brace        {push @pieces, substr ($$STRING, $LSPOS, call(oc_brace)) =~ /\S+/og}
-> get_pinport[1]  {return [@IMATCH_LIST, [@pieces]]}

I    {my @pieces}
LS   {push @pieces, map {split /(\s+)/o} grep {length} substr $$STRING, $IPOS, $LSPOS - $IPOS - length $LMATCH}
LE   {$IPOS = pos $$STRING}

oc_brace: /\{/ /\}/ -> oc_brace  -> oc_brace[1]  {return  $LSPOS - $IPOS - 1}
