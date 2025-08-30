(libview_links
 (each 
 (foreach col (if (defined $col) 
               (casematch $col
                 ^internal:             
                            (write $col (casematch $col "(?:library|cell|pin):" (subst {(?<=\@).*:} "" (col $col)) (subst {_none(\d+)_} $1 (col $col))) libview_link)                   
                 {^(?:library|cell|pin):} 
                            (write $col (subst "^.*?:" "" (col $col))  (concat libview_ (subst ":.*" "" (col $col))))
             
                 "^\w+:"    (write $col (subst {_none(\d+)_} $1 (col $col))  libview_other_group (nosubst 1))
                            (write $col $col default_center)
             
                )

               (write_force $col - default_center)
    )
  )
 )
)
