program:: I {print "==== program =====\n"}
-> if       {print "program: --> IF\n"; call(if)}
-> while    {print "program: --> WHILE\n"; call(while)}

if: /\bif\s*\(/   /\}/   I {print "==== if =====\n"}
-> then   {print "if: --> THEN\n"; call(then); return}

then: /\)\s*\{/ I {print "==== then =====\n"}
-> while   {print "then: --> WHILE\n"; call(while)}
-> if      {print "then: --> IF\n"; call(if)}
-> elsif   {print "then: --> ELSIF\n"; call(elsif); return}
-> else    {print "then: --> ELSE\n"; call(else); return}
-> if[1]   {print "then: CLOSING Brace\n"; return}

elsif: /\}\s*elsif\s*\(/ I {print "\n==== elsif =====\n"}
-> then   {print "\nelseif: --> IF_TOP_THEN\n"; call(then); return}

else: /\}\s*else\s*\{/ I {print "==== else =====\n"}
-> while   {print "else: --> WHILE\n"; call(while)}
-> if      {print "else: --> IF\n"; call(if)}
-> if[1]   {print "else: CLOSING BRACE SEEN\n"; return}


while: /\bwhile\s*\(/  /\}/       I {print "==== while =====\n"}
-> while_then  {print "while: --> WHILE_THEN\n"; call(while_then); return}

while_then: /\)\s*\{/ I {print "==== while_then =====\n"}
-> if        {print "while_then: --> IF\n"; call(if)}
-> while     {print "while_then: --> WHILE\n"; call(while)}
-> while[1]  {print "while_then: CLOSING BRACE SEEN\n"; return}
