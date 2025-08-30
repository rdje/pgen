(quick_omap2430c_dft
 (unshift Startpoint Endpoint "Frequency (Mhz)")
 (each (if {$ROW == 0}
   (foreach col (write $col $col header_wob))
   (else
    (write 0 0 input)
    (write 1 1 output)
    (write 2 2 frequency)
   )
  )
 )
)
