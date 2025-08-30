(xinstance_2ss
 (each (caseval {$ROW == 0}  (do
			      (write 0 (http (subst {:.+} {} (col 0)) (subst {.+:} {} (col 0))) white_on_blue_bold)
			      (write 1 (http (subst {:.+} {} (col 1)) (subst {.+:} {} (col 1))) black_on_greenyellow_b_s11_c)
			     )

		{$ROW == 1}  (foreach col (write $col $col header_wob))

			     (do
                              (write 0 0 black_on_beige)
			      (write 1 (unless (match 1 ^-) (http (subst {:.+} {} (col 1)) (subst {.+:} {} (col 1))) (subst {-:} {} (col 1)))  (unless (match 1 ^-) black_on_palegreen_c black_on_violet_wb_c))
			     )
  )  
 )
)

(xinstance_2ss_linkscript
 (unshift  Entity Architecture)
 (each  (if {$ROW == 0} (foreach col (write $col $col header_wob))
	    (do
	      (write 0 0 black_on_palegreen_c)
	      (write 1 1 (if (match 2 1$) black_on_paleturquoise_c black_on_orange_10))
	    )
  )
 )
)

(xdep_2ss
 (each (foreach col (unless (match $col {^ $}) (if (match $col ^top:) 
		                             (then
					       (set v  (subst {top:} {} (col $col)))
					       (set ap (subst {\|.+} {} (get v)))
					       (set ep (subst {.+\|} {} (get v)))
                                               ; archi
					       (if {'$ap' =~ /:/o} 
						   (do
						     (set label (subst {:.+} {} (get ap)))
						     (set html  (subst {.+:} {} (get ap)))

						     (write $col (http (get html) (get label))  black_on_paleturquoise_c)
						   ) 

						   (write $col (get ap) black_on_violet_wb_c)
					       )

                                               ; entity
					       (if {'$ep' =~ /:/o} 
						   (do
						     (set label (subst {:.+} {} (get ep)))
						     (set html  (subst {.+:} {} (get ep)))

						     (write (add (get col) 1) (http (get html) (get label))  black_on_greenyellow_b_s11_c)
						   ) 

						   (write (add (get col) 1) (get ep) black_on_orange_10)
					       )

		                             ) 
		                             

		                             (else
;-----------------------------------------------------------

					       (set v  (subst {node:} {} (col $col)))
					       (set ap (subst {\|.+} {} (get v)))
					       (set ep (subst {.+\|} {} (get v)))
                                               ; archi
					       (if {'$ap' =~ /:/o} 
						   (do
						     (set label (subst {:.+} {} (get ap)))
						     (set html  (subst {.+:} {} (get ap)))

						     (write $col (http (get html) (get label))  black_on_paleturquoise_c)
						   ) 

						   (write $col (get ap) black_on_violet_wb_c)
					       )

                                               ; entity
					       (if {'$ep' =~ /:/o} 
						   (do
						     (set label (subst {:.+} {} (get ep)))
						     (set html  (subst {.+:} {} (get ep)))

						     (write (add (get col) 1) (http (get html) (get label))  black_on_greenyellow_b_s11_c)
						   ) 

						   (write (add (get col) 1) (get ep) black_on_orange_10)
					       )


;-----------------------------------------------------------
		                             )
		      ) 

		      (write $col $col "")
	)
       )
 )
)
