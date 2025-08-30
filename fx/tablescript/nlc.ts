(nlc_link (unshift Category "Status Type" Count "Full Description")
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
     (write 0 0 black_on_bgreen_wb)
     (write 1 1 (casematch 1 "(?i)error" white_on_red_bold black_light_orange1_wb))
     (write 2 2 (casematch 2   -$   undefined_grey_wb  black_on_yellow_bbb))
     (write 3 3 black_on_light_blue2_wb)
    )
   )
 )
)


(nlc_CHK10 (unshift Library Cell)
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 black_on_beige_10)
      (write 1 1 black_on_lgreen)
    )
   )
 )
)

(nlc_CHK29 (unshift Library Cell Net)
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 black_on_beige_10)
      (write 1 1 black_on_lgreen)
      (write 2 2 black_on_light_blue3)
    )
   )
 )
)

(nlc_CHK50   (call nlc_CHK60))

(nlc_CHK52 (unshift Library Cell Instance "Forbidden Library" "Forbidden Cell")
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 black_on_beige_10)
      (write 1 1 black_on_lgreen)
      (write 2 2 black_on_light_blue3)
      (write 3 3 black_on_orange_10)
      (write 4 4 black_on_light_blue2)
    )
   )
 )
)

(nlc_CHK60 (unshift Library Cell Port)
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 black_on_beige_10)
      (write 1 1 black_on_lgreen)
      (write 2 2 black_on_orange_10)
    )
   )
 )
)



(nlc_R0 (unshift Library Cell)
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 black_on_beige_10)
      (write 1 1 black_on_lgreen)
    )
   )
 )
)

(nlc_R2 (unshift Net/Pin Cell)
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (if (match 0 NET:)
       (foreach col (write $col $col black_on_light_blue3))
       (else
        (write 0 0 white_on_purple)
        (write 1 1 black_on_lgreen)	
       )
    )
   )
 )
)

(nlc_R3.2 (unshift Port Master)
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
     (else
      (write 0 0 black_on_beige_10)
      (write 1 1 black_on_light_blue2)
     )
   )
 )
)

(nlc_R4 (unshift "Output Pin" Cell "Library Cell")
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 white_on_purple)
      (write 1 1 black_on_lgreen)
      (write 2 2 black_on_light_blue2)
    )
   )
 )
)

(nlc_R21.1 (unshift "Input port or Bidir Pin"  Instance  "Library Cell")
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 black_on_beige_10)
      (write 1 1 black_on_lgreen)
      (write 2 2 black_on_light_blue2)
    )
   )
 )
)

(nlc_R22 (unshift "Leaf Pin" Cell "Library Cell")
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 white_on_purple)
      (write 1 1 black_on_lgreen)
      (write 2 2 black_on_light_blue2)
    )
   )
 )
)

(nlc_R23.2 (unshift Net  Pin  Cell)
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 black_on_light_blue3)
      (write 1 1 white_on_purple)
      (write 2 2 black_on_lgreen)
    )
   )
 )
)
