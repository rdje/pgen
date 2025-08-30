(network
 (set column 1 color 0 red_on_yellow     ) 
 (set column 1 color 1 litegreen_1black  )
 (set column 1 color 2 black_light_orange)

 (set column 2 color 0 black_on_lgreen   ) 
 (set column 2 color 1 white_on_darkviolet)
 (set column 2 color 2 blue_on_liteblue_b)

 (set column 3 color 0 green_cells       ) 
 (set column 3 color 1 litepurple        )
 (set column 3 color 2 yellow_on_brown   )

 (set max_col_color 3)

 (each (foreach col 
         (if (defined $col) 
	   (write $col $col 
             (if {$col == 0} status_constrained
	          (casematch $col HS65_LS_CNHLSX17 (do
						      (set mod3_col (add (mod (sub (get col) 1) 3) 1))
						      (if (eq (get c_cg (get col))  "") 
	            				        (then
                                                          (set c_pt (get col) 0)
	            				          (set c_cg (get col) (col $col))
	            				          (set c_f  (get col) (get column (get mod3_col) color (get c_pt (get col))))
	            				        )

	            				        (else
	            				          (set n_cg (col $col))
	            				          (if (eq (get c_cg (get col)) (get n_cg))
	            				            (get c_f (get col))
	            				            (do
	            				             (set c_cg (get col) (get n_cg))
							     (set c_pt (get col) (mod (add (get c_pt (get col)) 1) (get max_col_color)))  
	            				             (set c_f  (get col) (get column (get mod3_col) color (get c_pt (get col))))
	            				            )
	            				          )
	            				        )
						       )
						     )

	            	     common: default_wonred
	            		     default
                 )
	     )
	   )

	   (write_force $col - default)
	 )
	)
 )
)
