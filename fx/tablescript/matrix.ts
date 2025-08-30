(matrix
 (each (foreach col
   (unless {$ROW < 6 && $col < 7} 
     (if (defined $col) (write $col $col (if {$ROW <= 5} ep_triplet
                                      (caseval {$col < 6}  (caseval 
                                                             {$col == 0} (concat (col $col)  _ path_type)
                                                             {$col == 1} path_group
                                                             {$col == 2} (eval {"@(2)" eq '-' ? 'undefined' : 'refclock_wb'})
                                                             {$col == 3} (eval {"@(3)" eq '-' ? 'undefined' : 'refclock_wb'})
                                                             {$col == 4} name_wb
                                                             {$col == 5} name_wb)
                                               {$col == 6} (casematch $col ^- negative_slack positive_slack) 
                                                           (casematch $col  "-\d+\.\d+\s+\(" violink_cell link_cell))
         )
        )

       (write_force $col  - undefined)
     )

     (write_force $col "" "")
   )
  )
 )
)
