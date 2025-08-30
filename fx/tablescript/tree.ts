(dirlist_2ss
  (each (foreach col (unless (match $col ^info:) (write $col (if (match $col ^leaf:) 
								  (http (subst {^\w+:} {} (col -1)) (subst {^leaf:}  {} (col $col)))
                                                                  (col $col)
                                                             ) 
                         (casematch $col 
                           ^node:           ss_node
                           ^leaf:           ss_leaf
                           ^notsupported:   ss_notsupported
                                            ""
                         )
                    )
                   )
  )
 )
)
