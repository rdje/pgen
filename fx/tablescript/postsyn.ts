(postsyn_link (unshift "Check Type" Category "Status Type" Count "Short Description" "Full Description")
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
     (write 0 0 (casematch 0 coverage  black_on_beige_10_wb black_on_light_blue2_wb))
     (write 1 1 black_on_bgreen_wb)
     (write 2 2 (casematch 2 "(?i)error" white_on_red_bold "(?i)warning" black_light_orange1_wb black_on_beige_10_wb))
     (write 3 3 (casematch 3   -$   undefined_grey_wb  black_on_yellow_bbb))
     (write 4 4 (casematch 4   -$   undefined_grey_wb  black_on_orange_10))
     (write 5 5 black_on_light_blue2_wb)
    )
   )
 )
)

(postsyn_C2 (unshift "Nonscan DFF" "Library Cell")
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 black_on_beige_10_wb)
      (write 1 1 black_on_light_blue2)
    )
   )
 )
)

(postsyn_C4   (unshift Clock)
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (write 0 0 refclock_wb)
   )
 )
)

(postsyn_C5  (call postsyn_C8))

(postsyn_C8   (unshift "Clock / Reset"  DFF1   DFF2)
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 black_on_beige_10)
      (write 1 1 black_on_orange_10)
      (write 2 2 black_on_light_blue3)
    )
   )
 )
)

(postsyn_C17  (unshift Clock  "Primary Output")
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 refclock_wb)
      (write 1 1 white_on_purple)
    )
   )
 )
)

(postsyn_C18  (call postsyn_C17))

(postsyn_C23  (unshift DFF  "Unstable DFF")
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 black_on_orange_10)
      (write 1 1 black_light_orange1_b)
    )
   )
 )
)

(postsyn_C24  (unshift "Nonclock PI"  "Unstable DFF")
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 black_on_beige_10)
      (write 1 1 black_light_orange1_b)
    )
   )
 )
)

(postsyn_C25  (unshift "Unstable DFF" PI DFF)
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 black_light_orange1_b)
      (write 1 1 black_on_beige_10)
      (write 2 2 black_on_orange_10)
    )
   )
 )
)

(postsyn_C26  (unshift "Clock / Reset" Clock "Stable DFF")
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 black_on_beige_10)
      (write 1 1 refclock_wb)
      (write 2 2 black_light_orange1_b)
    )
   )
 )
)

(postsyn_D1   (unshift  Clock  DFF)
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 refclock_wb)
      (write 1 1 black_light_orange1_b)
    )
   )
 )
)

(postsyn_D2   (unshift  DFF)
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 black_on_orange_10)
    )
   )
 )
)

(postsyn_D3   (unshift  DFF)
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 black_on_orange_10)
    )
   )
 )
)

(postsyn_D8   (unshift Clock)
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (write 0 0 refclock_wb)
   )
 )
)

(postsyn_D9   (unshift  DFF)
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 black_on_orange_10)
    )
   )
 )
)

(postsyn_D10   (unshift  Clock  "Data pin" DFF)
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 refclock_wb)
      (write 1 1 black_on_beige_10)
      (write 2 2 black_light_orange1_b)
    )
   )
 )
)

(postsyn_D15  (unshift  "Clock / Reset" DFF1   DFF2)
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 black_on_beige_10)
      (write 1 1 black_on_orange_10)
      (write 2 2 black_on_light_blue3)
    )
   )
 )
)

(postsyn_TEST-120  (unshift Cell "Library Cell")
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 black_on_lgreen)
      (write 1 1 (casematch 1 -  undefined_grey_wb black_on_light_blue2))
    )
   )
 )
  
)

(postsyn_TEST-121  (unshift Cell "Library Cell")
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 black_on_lgreen)
      (write 1 1 (casematch 1 -  undefined_grey_wb black_on_light_blue2))
    )
   )
 )
  
)

(postsyn_TEST-202  (unshift Cell "Library Cell")
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 black_on_lgreen)
      (write 1 1 (casematch 1 -  undefined_grey_wb black_on_light_blue2))
    )
   )
 )
  
)

(postsyn_TEST-451  (unshift Cell "Library Cell" "Output Pin")
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 black_on_lgreen)
      (write 1 1 (casematch 1 -   undefined_grey_wb black_on_light_blue2))
      (write 2 2 (casematch 2 -   undefined_grey_wb white_on_purple))
    )
   )
 )
  
)

(postsyn_TEST-504  (call  postsyn_TEST-505))

(postsyn_TEST-505  (unshift Cell Constant)
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 black_on_lgreen)
      (write 1 1 (casematch 1 -   undefined_grey_wb binary))
    )
   )
 )
  
)

(postsyn_S19  (unshift "Nonscan DFF" "Library Cell")
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 black_on_beige_10_wb)
      (write 1 1 black_on_light_blue2)
    )
   )
 )
)

(postsyn_S27  (unshift Latch "Clock Output")
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
    (else
      (write 0 0 black_on_light_blue3)
      (write 1 1 refclock_wb)
    )
   )
 )
)

