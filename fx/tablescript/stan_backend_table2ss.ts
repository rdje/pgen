(filterout_link_script
 (unshift Type Mode Corner "IO Mode" "Start Point" "End Point" "Path Group" "WNS (ns)" "# Violations")
 (each (foreach col (write $col $col (if {$ROW == 0} header_wob (if {$col >= 7}  (if {$col == 7}  (casematch 7 "@-" negative_slack positive_slack) count) 
                                                                                 (casematch 7 "@-" violink_cell link_cell))))))
)

(syncts_link_script
 ;(unshift Type "Path Group" "Launch Clock" "Capture Clock" "WNS L-Edge" "WNS C-Edge"  "Frequency -Mhz- (ns)" "WNS (ns)" "# Violations")
 (unshift Type "Path Group" "Launch Clock" "Capture Clock" "WNS L-Edge" "WNS C-Edge" "L-latency (ns)" "C-latency (ns)" "Skew (ns)" "Frequency -Mhz- (ns)" "WNS (ns)" "# Violations")
 (each (foreach col 
         (write $col $col (if {$ROW == 0} header_wob 
                                          (caseval {$col >= 10} (if {$col == 10}  (casematch 10 "@-" negative_slack positive_slack) count) 
					           {$col <  4}  (casematch 10 "@-" violink_cell link_cell)
						   {$col == 9}  (casematch 9 - undefined \? nofrequency vfrequency)
						   {$col == 4}  clockedge
						   {$col == 5}  clockedge_lrb 
						   {$col == 8}  (casematch 8 -$ undefined -\d neg_skew pos_skew)
						   {$col == 6}  l_latencies
						                r_latencies
                                          )
                          )
        )
  )
 )
)

(interface_link_script
 (unshift Type "IO Mode" "Path Group" "Launch Clock" "Capture Clock" "WNS L-Edge" "WNS C-Edge" "Frequency -Mhz- (ns)" "WNS (ns)" "# Violations")
 (each (foreach col 
         (write $col $col (if {$ROW == 0} header_wob 
                                          (caseval {$col >= 8}  (if {$col == 8}  (casematch 8 "@-" negative_slack positive_slack) count) 
					           {$col <  5}  (casematch 8 "@-" violink_cell link_cell)
						   {$col == 7}  (casematch 7 - undefined \? nofrequency vfrequency)
						   {$col == 6}  clockedge_lrb 
						                clockedge
                                          )
                          )
        )
  )
 )
)

(quick_dft_link_script
 (unshift Type Port "WNS (ns)" "# Violations")
 (each (foreach col (write $col $col (if {$ROW == 0} header_wob (if {$col >= 2}  (if {$col == 2}  (casematch 2 "@-" negative_slack positive_slack) count) 
                                                                             (casematch 2 "@-" violink_cell link_cell))))))
)

(eponsout_link_script
 (each (foreach col
   (write $col $col (if {$ROW == 0} (if {$col < 6} header_wob ep_triplet)
                                    (if {$col < 6} (caseval {$col == 5} (join "" (casematch 5 "-\d+\.\d+\s+\(" negative positive) _slack) 
                                                            {$col == 0} line_index 
                                                                        (join "" (casematch 5 "-\d+\.\d+\s+\("  vio "") link_cell))

                                                   (casematch $col ^-$ undefined "-\d+\.\d+\s+\(" vcelldata margin_number))
    )
   )
  )
 )
)
