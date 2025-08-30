
(consolidated_rep_skew
 (unshift  Mode IOMode Corner "File Path" StartPoint "StartPoint Clock" "SPC Edge" EndPoint "EndPoint Clock" "EPC Edge" "Arrival Time")
 (each (if {$ROW == 0}
   (foreach col (write $col $col header_wob))
   (do
    (call rpt_consolidate_common)
    (write 10 arrival_time litepurple)
   )
  )
 )
)

(skew_summary
 (each (foreach col (
       if (defined $col) (write $col $col (casematch $col Skew                header_wob 
                                                          REF                 white_on_darkviolet
                                                          DATA                white_on_dblue
                                                          "-?\d+(?:\.\d+)?"   (caseval {$col == 0} black_light_orange1
							                               {$col == 3} black_light_orange 
										       {$col == 1} (if {@(1) > @(0)} black_on_bgreen white_on_red_bold)
										       {$col == 2} (if {0 < @(2) && @(2) < @(3)} black_on_bgreen white_on_red_bold) 
									      )

									      undefined

					  )
			 )

        (write_force $col "" undefined_format)
   )
  )
 )
)

(skew_summary_link
 (each (foreach col (write $col $col (if {$col == 0} white_on_blue_bold_11 (casematch $col max black_on_orange_bold_11 black_on_beige_bold_11)))))
)

(skew_modetype
 (each (foreach col (
    write $col $col (caseval {$col == 0} white_on_blue_10
                             {$col == 1} (casematch $col max black_on_orange_10 black_on_beige_10)
			     {$col == ($MAXCOL - 1)} black_light_orange_b
			     {$col ==  $MAXCOL}      black_light_orange1_b
			     {$col == ($MAXCOL - 2)} (casematch $col fall black_on_light_blue2 black_on_blue)
                             black_on_lgreen
			     
	            )
   )
  )
 )
)
