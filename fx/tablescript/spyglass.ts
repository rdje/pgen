(sg_2ss
 (each (caseval {$ROW == 0} (foreach col (write $col $col black_on_greenyellow_b_s11))
                {$ROW == 1} (foreach col (write $col $col header_wob))

                            (casematch 1 
	                     "(?i)error"   (do (write_force 0 " " default_noborder) 
	                                       (foreach col (write (add (get col) 1) $col (if "$col == 4 || $col == 7" white_on_red_bold_nocenter       white_on_red_bold      ))) 
	                                       (write_force 9 " " default_noborder)) 

                             "(?i)warning" (do (write_force 0 " " default_noborder) 
	                                       (foreach col (write (add (get col) 1) $col (if "$col == 4 || $col == 7" black_light_orange1_wb_nocenter  black_light_orange1_wb ))) 
	                                       (write_force 9 " " default_noborder))

	                     "(?i)info"    (do (write_force 0 " " default_noborder) 
	                          	     (foreach col (write (add (get col) 1) $col (if "$col == 4 || $col == 7" black_on_beige_10_wb_nocenter    black_on_beige_10_wb   ))) 
	                          	     (write_force 9 " " default_noborder))

	                          	 (do (write_force 0 " " default_noborder) 
	                          	     (foreach col (write (add (get col) 1) $col (if "$col == 4 || $col == 7" black_on_light_blue2_wb_nocenter black_on_light_blue2_wb))) 
	                          	     (write_force 9 " " default_noborder))
	                    )

       )
 )
)

