dtree::								I {print "Decision TOP\n"}
 -> testcontrol
 -> inline_dt_definition
 -> state_transition
 -> dtree_call
 -> reg_assignment_lhs
 -> identifier

testcontrol: 		/\(\?/ /\)/				I {print "(testcontrol) -I- Entering\n"}
 -> testcontrol	
 -> inline_dt_definition
 -> if_binary
 -> if_vector
 -> reg_assignment_lhs
 -> identifier
 -> state_transition
 -> dtree_call
 -> group
 -> testcontrol[1]						{print "(testcontrol) -I- Leaving\n"; return 1}
 
group: 			/\(/ /\)/				I {print "(group) -I- Entering\n"}
 -> group
 -> identifier
 -> logical_operator
 -> group[1]							{print "(group) -I- Leaving\n"; return 1}

identifier: 		/[a-zA-Z_]\w*/				I {print "(identifier)($IMATCH)\n"}
if_binary: 		/[01]\s*:/				I {print "(if_binary)($IMATCH)\n"}
if_vector: 		/[!=><]+\d+\s*:/			I {print "(if_vector)($IMATCH)\n"}
reg_assignment_lhs: 	/\w+(?:\[\d+(?::\d+)?\])?\s*\=/ 	I {print "(reg_assignment)($IMATCH)\n"}

state_transition:	/\=\>\s*\w+/				I {print "(state_transition)($IMATCH)\n"}
dtree_call:		/->\s*\w+(?:\(.+\))?/			I {print "(dtree_call)($IMATCH)\n"}
logical_operator:	/\&|\||!/				I {print "(logical_operator)($IMATCH)\n"}

inline_dt_definition:	/\(:/ /\)/				I {print "(inline_dt_definition) -I- Entering\n"}
 -> inline_dt_definition
 -> testcontrol
 -> state_transition
 -> dtree_call
 -> reg_assignment_lhs
 -> identifier
 -> inline_dt_definition[1]					{print "(inline_dt_definition) -I- Leaving\n"; return 1}
