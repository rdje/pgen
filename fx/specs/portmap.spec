# Multi branching IS NOT to be used when connecting INPUT Pins of component
# But ONLY for having the same output pin of a component instance driving
# more than one SINK, i.e, internal signal (OUT-pin -------< IN-pins) and OUT-port(s)
portmap::
-> bare_bit_slice .push
-> concatenation  .push

LX {return @portmap == 1 ? $portmap[0] : ['?multi:', [@portmap]]}


concatenation: /\{/ /\}/
-> concatenation  .push
-> bare_bit_slice .push
-> concatenation[1]    {return ['?concat:', [@concatenation]]} 

bare_bit_slice: /([[:alpha:]]\w*)(?:\[(?:(\d+)(?::(\d+))?|(\?[[:alpha:]]\w+))\])?|(?i)(0x[0-9a-f]+|0b[01]+|\d+\'\d+)/ I {
	my $mcnt = @IMATCH_LIST;
	#say "bare_bit_slice: <@IMATCH_LIST>";
	return ['?'.($mcnt == 1 ? ($IMATCH_LIST[0] ~~ /^\d/io ? 'constant' : 'bare') : ($mcnt == 2 ? 'bit' : 'slice')).':', [@IMATCH_LIST]]
}
