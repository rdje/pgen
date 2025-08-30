
(wrapgen
 (each (foreach col (if (defined $col) 
		      (write $col $col (casematch $col in:  input out: output default)) 
		      (write_force $col - default)
		    )
       )
 )
)
