(seview
 (unshift StartPoint EndPoint)
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
         (foreach col (write $col $col (casematch $col removed: litered_white_aleft / pin_wrb port_wrb)))
	)
       )
)

(seview_n_adjust
 (unshift StartPoint EndPoint Adjust)
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
         (foreach col (write $col $col (casematch $col removed: litered_white_aleft "^\d+(?:\.\d+)?$" margin_number / pin_wrb port_wrb)))
	)
       )
)
