(raw
 (each (foreach col (if (defined $col) (write $col $col default) (write_force $col - default))))
)

(raw_w_header
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
          (foreach col (if (defined $col) (write $col $col default) (write_force $col - default)))
      )
 )
)

