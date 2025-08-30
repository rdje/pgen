; links
(qcflow_links
 (each (write 0 0 links))
)

(qcflow_ctsinfo
 (unshift "Related Pin" Ctsmin Ctsmax "Excel2HM CTSmin" "Excel2HM CTSmax")
 (each (if {$ROW == 0}
   (foreach col (write $col $col header_wob))
   (else
    (write 0 0 qcflow_refclock_green)
    (write 1 1 (eval {"@(1)" =~ /V:/ ? 'vcts' : 'ctsmin'}))
    (write 2 2 (eval {"@(2)" =~ /V:/ ? 'vcts' : 'ctsmax'}))
    (write 3 3 min_budget)
    (write 4 4 max_budget)
   )
  )
 )
)

; piname - lib direction - xcel2hm direction
(qcflow_"Pin Direction mismatch between LIB vs XCEL2HM"
 (unshift Pin "Lib Direction" "Excel2hm Direction")
 (each  (if {$ROW == 0}
   (foreach col (write $col $col header_wob))
   (else
    (write 0 0 qcflow_name)
    (write 1 1 (eval {"@(1)" =~ /input/  ? 'qcflow_input' : 'qcflow_output'}))
    (write 2 2 (eval {"@(2)" =~ /(?i)in/ ? 'qcflow_input' : 'qcflow_output'}))
   )
  )
 )
)

; piname - direction - related_pin - timing_type 
(qcflow_"Timing Arcs ONLY defined in the LIB file, though missing from XCEL2HM"
 (unshift Pin Direction "Related Pin" "Timing Type")
 (each (if {$ROW == 0}
   (foreach col (write $col $col header_wob))
   (else
    (write 0 0 qcflow_name)
    (write 1 1 (casematch 1 input qcflow_input qcflow_output))
    ; OR (write 1 1 (eval {"@(1)" =~ /input/  ? 'qcflow_input' : 'qcflow_output'}))
    (write 2 2 qcflow_refclock_green)

    (write 3 3 (casematch 3 ris  rise_check
                            fall fall_check
                            combinational combinational
                            pulse    pulse_type
        		    default_center
     )
    )
   )
  )
 )
)

; piname - related_pin - timing_type 
(qcflow_"Timing Type not support in XCEL2HM"
 (unshift Pin "Reference Clock" "Timing Type")
 (each (if {$ROW == 0}
   (foreach col (write $col $col header_wob))
   (else
    (write 0 0 qcflow_name)
    (write 1 1 qcflow_refclock_green)
    (write 2 2 (casematch 2 ris rise_check fall fall_check combinational combinational default_center))
   )
  )
 )
)

; piname - direction - related_pin
(qcflow_"Port used as *related_pin* in the LIB file but not defined as clock in XCEL2HM"
 (unshift Pin Direction "Reference Clock")
 (each (if {$ROW == 0}
   (foreach col (write $col $col header_wob))
   (else
    (write 0 0 qcflow_name)
    (write 1 1 (casematch 1 input qcflow_input qcflow_output))
    (write 2 2 qcflow_refclock_green)
   )
  )
 )
)

; piname - direction - related_pin - (max|min)_budget
(qcflow_"Missing budget in XCEL2HM"
 (unshift Pin Direction "Reference Clock" "Budget Type")
 (each (if {$ROW == 0} ; Same meaning as above
   (foreach col (write $col $col header_wob))
   (else
    (write 0 0 qcflow_name)
    (write 1 1 (casematch 1 input qcflow_input qcflow_output))
    (write 2 2 qcflow_refclock_green)
    (write 3 3 (casematch 3 min_budget min_budget max_budget))
   )
  )
 )
)

; piname - direction - related_pin - (max|min)_budget
(qcflow_"Non negative HOLD budget when expressed w.r.t a reference clock"
 (unshift Pin Direction "Reference Clock" "Xcel2hm Budget")
 (each (if {$ROW == 0} 
   (foreach col (write $col $col header_wob))
   (else
    (write 0 0 qcflow_name)
    (write 1 1 (casematch 1 input qcflow_input qcflow_output))
    (write 2 2 qcflow_refclock_green)
    (write 3 3 xcel2hm_info)
   )
  )
 )
)

; related_pin
(qcflow_"Port used as *related_pin* in LIB file but not defined as clock in XCEL2HM"
 (unshift "Pin as Related Pin")
 (each (write 0 0 (if {$ROW == 0} header_wob name_center)))
)

; pin name
(qcflow_"(LIB vs XCEL2HM) Port with NO timing information in the LIB file"
 (unshift Pin)
 (each (write 0 0 (if {$ROW == 0} header_wob name_center)))
)

; pin name
(qcflow_"(LIB) Port(s) with NO Timing Information"
 (unshift Pin)
 (each (write 0 0 (if {$ROW == 0} header_wob name_center)))
)

; piname - direction - related_pin - xxx_budget - timing_type - timinginfoname - real_budget - Xcel2hm_budget - slack 
(qcflow_"Violating Timing Arcs"
 (unshift Index Pin  Direction  "Related Pin"  "Budget Type"  "Timing Type"  "Arc Type" "Real Budget" "Xcel2hm Budget" Slack)
 (each (if {$ROW == 0}
   (foreach col (write $col $col header_wob))
   (else
    (write 0 0 line_index)
    (write 1 1 qcflow_name)
    (write 2 2 (casematch 2 input qcflow_input qcflow_output))
    (write 3 3 qcflow_refclock_green)
    (write 4 4 (casematch 4 min_budget min_budget max_budget))
    (write 5 5 (casematch 5 ris rise_check fall fall_check combinational combinational default_center))
    (write 6 6 (casematch 6 rise rise_timing fall_timing))
    (write 7 7 margin_number)
    (write 8 8 xcel2hm_info)
    (write 9 9 negative_slack)
   )
  )
 )
)

(qcflow_common1
 (write 0 0 qcflow_name)
 (write 1 1 (casematch 1 input qcflow_input qcflow_output))
 (write 2 2 qcflow_refclock_green)
 (write 3 3 (casematch 3 min_budget min_budget max_budget))
 (write 4 4 (casematch 4 ris rise_check fall fall_check combinational combinational default_center))
 (write 5 5 (casematch 5 rise rise_timing fall_timing))
)

; piname - direction - related_pin - xxx_budget - timing_type - timinginfoname - CTS LIB - noCTS LIB 
(qcflow_"Timing Arc Table with Size mismatch between CTS LIB and NoCTS LIB"
 (unshift Pin  Direction  Related_pin  "Budget Type"  "Timing Type"  "Arc Type" "CTS LIB" "noCTS LIB")
 (each (if {$ROW == 0}
   (foreach col (write $col $col header_wob))
   (else
    (call qcflow_common1)
    (write 6 6 gl_info)
    (write 7 7 xcel2hm_info)
   )
  )
 )
)

; piname - direction - related_pin - xxx_budget - timing_type - timinginfoname 
(qcflow_"Timing ARCs missing in the noCTS LIB vs CTS LIB"
 (unshift Pin  Direction  Related_pin  "Budget Type"  "Timing Type"  "Arc Type")
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob)) (call qcflow_common1)))
)

; Timing info values table
(qcflow_vvalues
 (each (foreach col (write $col $col (casematch $col internal: violating_timinfo margin_number))))
)

; Extracted CTS
(qcflow_xcts
 (unshift just_a_place_holder)
 (each (foreach col (write $col $col (if {$ROW == 0} header_wob (casematch $col ^- violating_timinfo margin_number)))))
)

; ARC info
(qcflow_arcinfo
 (each (foreach col (write $col $col margin_number)))
)

; Real budgets
(qcflow_realbudgets
 (each (foreach col (write $col $col (casematch $col % margin_number default_center))))
)

; piname - direction - related_pin - timing_type 
(qcflow_"Port declared as Clock Gating Check Enable in LIB file"
 (unshift Pin  Direction "Related Pin" "Timing Type")
 (each (if {$ROW == 0}
   (foreach col (write $col $col header_wob))
   (else
    (write 0 0 qcflow_name)
    (write 1 1 (casematch 1 input qcflow_input qcflow_output))
    (write 2 2 qcflow_refclock_green)
    (write 3 3 (casematch 3 ris rise_check fall fall_check combinational combinational default_center))
   )
  )
 )
)

(qcflow_"Port definition missing from XCEL2HM"
 (unshift Pin  Direction)
 (each (if {$ROW == 0}
   (foreach col (write $col $col header_wob))
   (else
    (write 0 0 qcflow_name)
    (write 1 1 (casematch 1 input qcflow_input qcflow_output))
   )
  )
 )
)

(qcflow_"Incorrectly specified Bus range in XCEL2HM"
 (unshift "LIB Bus Name")
 (each (write 0 0 (if {$ROW == 0} header_wob qcflow_name)))
)

(qcflow_"Bus range information not specified in XCEL2HM"
 (unshift "Bus Name")
 (each (write 0 0 (if {$ROW == 0} header_wob qcflow_name)))
)

(qcflow_"Bus Slice defined in the LIB file not belonging to the range defined in XCEL2HM"
 (unshift "LIB Slice" "Xcel2hm Bus Spec")
 (each (if {$ROW == 0}
   (foreach col (write $col $col header_wob))
   (else
    (write 0 0 qcflow_name)
    (write 1 1 bus_range)
   )
  )
 )
)

(qcflow_"Port used as a clock in XCEL2HM but not defined as clock in XCEL2HM"
 (unshift "Clock(s) ?")
 (each (write 0 0 (if {$ROW == 0} header_wob qcflow_name)))
)

(qcflow_"(XCEL2HM vs GUIDELINE) Clock Edge Mismatch"
 (unshift Pin Direction "Reference Clock" "Xcel2hm Edge" "Guideline Edge")
 (each (if {$ROW == 0}
   (foreach col (write $col $col header_wob))
   (else
    (write 0 0 qcflow_name)
    (write 1 1 (casematch 1 "(?i)in" qcflow_input qcflow_output))
    (write 2 2 qcflow_refclock_green)
    (write 3 3 xcel2hm_info)
    (write 4 4 gl_info)
   )
  )
 )
)

(budget_check_macro
 (if {"$fanx" eq ""}
  (if {"$uicomment" eq ""} 
   (unshift Pin Direction "Reference Clock" "Freqency -Mhz- (ns)" "Budget Type" "Xcel2hm % (ns)" "GL % (ns)"         Interface Regex)
   (unshift Pin Direction "Reference Clock" "Freqency -Mhz- (ns)" "Budget Type" "Xcel2hm % (ns)" "GL % (ns)" Comment Interface Regex)
  )

  (if {"$uicomment" eq ""} 
   (unshift Pin Direction "Reference Clock" "Freqency -Mhz- (ns)" "Budget Type" "Xcel2hm % (ns)" "GL % (ns)" Fanx         Interface Regex)
   (unshift Pin Direction "Reference Clock" "Freqency -Mhz- (ns)" "Budget Type" "Xcel2hm % (ns)" "GL % (ns)" Fanx Comment Interface Regex)
  )
 )

 (each (if {$ROW == 0}
   (foreach col (write $col $col header_wob))
   (else
    (write 0  0  qcflow_name)
    (write 1  1  (casematch 1 "(?i)in" qcflow_input qcflow_output))
    (write 2  2  qcflow_refclock_green)
    (write 3  3  qcflow_frequency)
    (write 4  4  (casematch 4 "(?i)in" min_budget max_budget))
    (write 5  5  xcel2hm_info)
    (write 6  6  gl_info)

    (casematch 9 ^\+$
     (casematch 10 ^\+$
      (then
       (write 7 7  (casematch 7 NOT_COVERED     gl_notcovered gl_interface))
       (write 8 8  (casematch 8 not_applicable  gl_notcovered gl_regex))
      )

      (else
       (write 7 10 qcflow_comment)
       (write 8 7  (casematch 7 NOT_COVERED     gl_notcovered gl_interface))
       (write 9 8  (casematch 8 not_applicable  gl_notcovered gl_regex))
      )
     )

     (else
      (write 7  9  (casematch 9 ^-$ undefined internal: qcflow_fanxlink  qcflow_1ep))

      (casematch 10 ^\+$
       (then
        (write 8 7  (casematch 7 NOT_COVERED     gl_notcovered gl_interface))
        (write 9 8  (casematch 8 not_applicable  gl_notcovered gl_regex))
       )

       (else
        (write 8  10 qcflow_comment)
        (write 9  7  (casematch 7 NOT_COVERED     gl_notcovered gl_interface))
        (write 10 8  (casematch 8 not_applicable  gl_notcovered gl_regex))
       )
      )
     )
    )
   )
  )
 )
)

(qcflow_"(XCEL2HM vs GUIDELINE) Budget Check: Failed"   (call budget_check_macro))
(qcflow_"(XCEL2HM vs GUIDELINE) Budget Check: Passed"   (call budget_check_macro))

(qcflow_"(XCEL2HM vs GUIDELINE) Ports not defined in the GUIDELINE"
 (unshift "Ports not in Guideline")
 (each (write 0 0 (if {$ROW == 0} header_wob qcflow_name)))
)

; Several matches
(qcflow_"(XCEL2HM vs GUIDELINE) Ports With Multiple Matches in the GUIDELINE"
 (unshift Pin "Interface(s)...")
 (each (foreach col (write $col $col (caseval {$ROW == 0} header_wob (caseval {$col == 0} qcflow_name gl_interface)))))
)

(qcflow_ctsinfo
 (unshift  Index Pin  Direction  "Related Pin"  "Budget Type"  "Timing Type"  "Arc Type")
 (each (if {$ROW == 0}
   (foreach col (write $col $col header_wob))
   (else
    (write 0 0 line_index)
    (write 1 1 qcflow_name)
    (write 2 2 (casematch 2 input qcflow_input qcflow_output))
    (write 3 3 qcflow_refclock_green)
    (write 4 4 (casematch 4 min_budget min_budget max_budget))
    (write 5 5 (casematch 5 ris rise_check fall fall_check combinational combinational default_center))
    (write 6 6 (casematch 6 rise rise_timing fall_timing))
   )
  )
 )
)


; piname - direction - related_pin - xxx_budget - timing_type - timinginfoname
(qcflow_port_2_xcel2hm_budget
 (each 
  (write 0 0  qcflow_name)
  (write 1 2  (casematch 2 "(?i)in" qcflow_input qcflow_output))
  (write 2 7  qcflow_refclock_green)
  (write 3 8  rise_timing)
  (write 4 10 min_budget)
  (write 5 11 max_budget)
 )
)

; piname - direction - related_pin - xxx_budget - timing_type - timinginfoname - comment
(qcflow_port_2_xcel2hm_budget_w_comment
 (each 
  (write 0 0  qcflow_name)
  (write 1 2  (casematch 2 "(?i)in" qcflow_input qcflow_output))
  (write 2 7  qcflow_refclock_green)
  (write 3 16 qcflow_frequency)
  (write 4 8  rise_timing)
  (write 5 17 min_budget)
  (write 6 18 max_budget)
  (write 7 12 qcflow_comment)
  (caseval {'@(15)' !~ /^\+$/o} (write 8 15 (casematch 15 ^-$ undefined internal: qcflow_fanxlink  qcflow_1ep)))
 )
)



(qcflow_"(XCEL2HM vs GUIDELINE) Ports with NO reference clock"
 (unshift Pin Direction)
 (each (if {$ROW == 0}
   (foreach col (write $col $col header_wob))
   (else
    (write 0 0 qcflow_name)
    (write 1 1 (casematch 1 "(?i)in" qcflow_input qcflow_output))
   )
  )
 )
)

(qcflow_"(XCEL2HM vs GUIDELINE) Ports with a reference clock but with NO min & max budgets"
 (unshift Pin Direction)
 (each (if {$ROW == 0}
   (foreach col (write $col $col header_wob))
   (else
    (write 0 0 qcflow_name)
    (write 1 1 (casematch 1 "(?i)in" qcflow_input qcflow_output))
   )
  )
 )
)

(qcflow_"(XCEL2HM vs GUIDELINE) No Guideline Budget defined"
 (unshift Pin Direction "Budget Type" GL)
 (each (if {$ROW == 0}
   (foreach col (write $col $col header_wob))
   (else
    (write 0 0 qcflow_name)
    (write 1 1 (casematch 1 "(?i)in"  qcflow_input qcflow_output))
    (write 2 2 (casematch 2 "(?i)max" max_budget min_budget))
    (write 3 3 gl_info)
   )
  )
 )
)
