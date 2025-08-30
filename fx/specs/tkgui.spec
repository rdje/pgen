sub_gui_list:: 
 I {my @sub_guis}
 -> sub_gui	{push @sub_guis, call(sub_gui)}
 -> comment	{next}

 LX {return {@sub_guis}}

sub_gui: /(\S+)\s+\{/   /\}/ 	
I {
 my ($subgui_name) = @IMATCH_LIST;
 print "Found a SUB GUI entry point <$subgui_name>\n";
}

 -> curlyb
 -> sub_gui[1]	  {return ($subgui_name => '('.substr($$STRING, $IPOS, $LSPOS - $IPOS - 1).')')}

curlyb: /\{/ /\}/
 -> curlyb
 -> comment	{next}
 -> curlyb[1]   {return} 

comment: /#.*\n/
