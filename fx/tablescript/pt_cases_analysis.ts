(port
 ; Adding header
 (unshift Port Direction "Case Value")
 (each (if {$ROW == 0}
   (foreach col (write $col $col header_wob))
   (else
    (write 0 2 name)
    (casematch 3 "(?i:inout)" (write 1 3 inout)
                 {(?i:in)$}   (write 1 3 input)
		 "^(?i:out)"  (write 1 3 output)
    )

    (write       2 4     (casematch 4 - dash binary))
   )
  )
 )
)

(pin
 ; Adding header
 (unshift Module "Module Path" "Pin Relative Path" Direction "Case Value")
 (each (if {$ROW == 0}
   (foreach col (write $col $col header_wob))
   (else
    (write 0 0 name)
    (write 1 1 default)
    (write 2 2 pin_relpath)
    (casematch 3 "(?i:inout)" (write 3 3 inout)
                 {(?i:in)$}   (write 3 3 input)
                 "^(?i:out)"  (write 3 3 output)
		 "^(?i:all)"  (write_force 3 - nodirection)
    )

    (write       4 4     (casematch 4 - dash binary))
   )
  )
 )
) 
