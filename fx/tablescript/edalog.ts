
(edalog_link (unshift Type Category Count)
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
     (write 0 0 (casematch 0 "(?i)error" white_on_red_bold "(?i)warning" black_light_orange1_wb black_on_beige_10_wb))
     (write 1 1 black_on_bgreen_wb)
     (write 2 2 (casematch 2   -$   undefined_grey_wb  black_on_yellow_bbb))
    )
   )
 )
)

(edalog_script
  (each (foreach col (do 
      (write 0 0 black_light_orange)
      (write 1 1 black_on_light_blue3)
    )
   )
  )
)
