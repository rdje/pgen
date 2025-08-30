(default_lof_driver_summary
  (each (foreach col (write $col $col (casematch $col internal: asimple_link asimple_value))))
)

(captions
 (each
  (write 0 0 caption_index)
  (write 1 1 caption_name)
 )
)

(clockinfo
  (each (foreach col (do
    (if "$col == 0" (write 0 0 name))
    (if "$col == 1" (write 1 1 number))
    (if "$col == 2" (write_force 2 " " (casematch 2 "(?i:propagated)" propagated ideal)))

    (if "$col == 5" 
      (if (match $col -) (write_force 3 " " default) (write 3 5 clocksrc))
    )

    (caseval {$col == 6} (write 4 6   (casematch 6 "(?i:inout)" inout {(?i:in)$} input "^(?i:out)" output))
             {$col == 7} (write 5 7   (casematch 7 "(?i:pin)" pin port))
             {$col == 3} (write 6 3   number)
             {$col == 4} (write 7 4   hold_uncertainty)
	     {$col >= 8} (write $col $col (casematch $col \/ isit_mc true_potential_mc))
    )
   )
  )
 )
)


(consolidated_rep
 (unshift  "Timing Path" StartPoint "StartPoint Clock" "SPC Edge" EndPoint "EndPoint Clock" "EPC Edge" "Slack (ns)")
 (each (if {$ROW == 0}
   (foreach col (write $col $col header_wob))
   (call rpt_consolidate_common)
  )
 )
)

(rpt_consolidate_common
 (linemap   reportiming)
 (write 0 (concat http://  (eval Global->http_hostport) /cgi-bin/getpath.cgi? (col query_string))  url)
 (write 1 startpoint              name)
 (write 2 startpoint_clock        (if (eq (col endpoint_clock) -)  undefined  refclock))
 (write 3 startpoint_clock_edge   clockedge)
 (write 4 endpoint                name)
 (write 5 endpoint_clock          (if (eq (col endpoint_clock) -)  undefined  refclock))
 (write 6 endpoint_clock_edge     clockedge)
 
 ; Slack
 (casematch slack "^internal:.+\@\d+(?:\.\d+)?"  (write       9 slack positive_slack_wlink)
                  "^internal:.+\@-"              (write       9 slack negative_slack_wlink)
                  "^\d+(?:\.\d+)?"               (write       9 slack positive_slack      )
                  "^-"                           (write       9 slack negative_slack      )
                  "(?i:infinity)"                (write_force 9 - unconstrained           ))
 
)

(tcfix_check
 (each (foreach col (if {$ROW == 0}
    (write $col $col name) 
    (casematch $col ^good:           (write       $col $col positive_slack)
                     ^bad:           (write       $col $col negative_slack)
                     "(?i)infinity"  (write_force $col ?    undefined)
                     ^-$             (write_force $col -    undefined)
    )
   )
  )
 )
)

(clock_margin
 (each (foreach col (if "$ROW == 0"
    ; First Row #0
    (if "$col == 0" (write_force $col " " undefined) (write $col $col refclock))

    ; Other Rows #1 to #last
    (if "$col == 0"
     (write $col $col refclock)
     (unless (defined $col)
      (write_force      $col - undefined)
      (write  $col $col (casematch $col ^-$  undefined "-\d+"  white_on_red default_green))
     )
    )
   )
  )
 )
)


(stan_frequency_summary
 (each (foreach col (casematch $col  \+            (write_force $col " "  undefined    )
                                     \?            (write       $col $col red_on_yellow)
        			     ^\d+\.\d+$    (write       $col $col voltage      )
                                     ^\d+$         (write       $col $col frequency    )
                                     [a-zA-Z_]+    (write       $col $col path_type    ))
  )
 )
)

(stan_nopath_check
 (unshift "Port Name" Direction Corner Mode IOMode "Path Type")
 (each (if {$ROW == 0}
   (foreach col (write $col $col header_wob))
   (else
    (write 0 0 portname)
    (write 1 5 (casematch 5 in input output))
    (write 2 1 voltage)
    (write 3 2 nopath_mode)
    (write 4 3 consolid_iomode)
    (write 5 4 path_type)
   )
  )
 )
)

(stan_frequency_detailed
 (each (foreach col (casematch $col /           (write $col $col       path_type)
                                    ^\d\.\d+$   (write $col $col       voltage)
                                    io_mode     (write $col $col       consolid_iomode)
                                    internal    (write $col $col       margin_number)
                                    \+          (write_force $col " "  undefined)
                                    ^-$         (write $col $col       white_on_red_bold)
                                    \?          (write $col $col       black_on_purple)
                                    "\("        (write $col $col       white_on_black_bold))
  )
 )
)

(stan_dmeasures
 (each (foreach col 
   (if {$ROW == 0}
    (casematch  $col \+ (write_force $col " " undefined) (write  $col $col  dm_corner))
    (casematch  $col "^[A-Z]+[-A-Z]*"            (write $col $col       dm_segment)
                    "^(?:min|max)"               (write $col $col       dm_minmax)
                    "internal:|^\d+(?:\.\d+)?"   (write $col $col       margin_number)
                    ^-$                          (write_force $col -    undefined)
                                                 (write       $col $col consolid_iomode))
   )
  )
 )
)

(tssio
 (each (foreach col 
   (if {$ROW == 0}
    (write $col $col header_wob)
    (if {$col <= 3}
     (then
      (write 0 0 name)
      (write 1 1 consolid_iomode)
      (write 2 2 port)
      (write 3 3 (casematch 3 - undefinedrefclock_green))
     )

     (casematch $col internal:
       (write $col $col margin_number)
       (casematch $col ^-$ 
        (if {(($col - 4) % 16 == 0) || (($col - 12) % 16 == 0)}
         (if {($col - 4) % 16 == 0} 
           (caseval {$col == 4}  (write $col $col undefined_2black)
                    {$col == 20} (write $col $col undefined_2blue)
                    {$col == 36} (write $col $col undefined_2red)
           )

           (write $col $col undefined_1black)
         )

         (write $col $col undefined)
        )
        
        (caseval
         {($col - 4) % 16 == 0} (caseval {$col == 4}  (write $col $col litegreen_2black)
                                         {$col == 20} (write $col $col litegreen_2blue)
                                         {$col == 36} (write $col $col litegreen_2red))

         {($col - 12) % 16 == 0} (write $col $col litegreen_1black)

         ; Cycle type
	 {($col - 5) % 4 == 0} (casematch $col ^H$ (write $col $col red_on_liteblue_b) 
		                                   (write $col $col blue_on_liteblue_b))
         
         ; TSS info
         {($col - 6) % 4 == 0} (write $col $col litepurple)
        )
       )
     )
    )
   )
  )
 )
)

(qcsummary
 (each (foreach col (do
    (if {$ROW == 0} 
     (casematch $col \+
      (write_force $col " " default)
      (write $col  $col consolid_corner)
     ) 
    )

    (if {$ROW == 1}
     (if {$col <= 5}
      (write $col $col header_wob)
      (caseval {$col == 6}  (write 7  $col header_wob)
               {$col == 7}  (write 9  $col header_wob)
               {$col == 8}  (write 6  $col header_wob)
               {$col == 9}  (write 8  $col header_wob)
               {$col == 10} (write 10 $col header_wob)
               {$col == 11} (write 11 $col header_wob))
     )
    )
    
    (if {$ROW >  1} 
     (if (match $col ^[\+\-]$)
      (if (match $col \+)
       (write_force $col " "  blue_on_liteblue_b)
       (if {$col == 11}
        (write_force $col " "  comment)
        (write       $col $col undefined)
       )
      )
      
      (caseval
        {$col == 0}  (write $col $col (casematch $col in input output))
        {$col == 1}  (write $col $col name)
        {$col == 2}  (do
                       (if (match $col "(?i)^constrained")                                   (write $col $col status_constrained))
                       (if (match $col "(?i)(?:unconstrained|not_constrained|clock_gating)") (write $col $col status_uconstrained))
                      )
        
        {$col == 3 || $col == 4}  (write $col $col margin_number)

        {$col == 5}  (casematch  $col "^\d+(?:\.\d+)?" (write $col $col "positive_slack")
                                      ^-               (write $col $col "negative_slack")
                     )
        
        {$col == 6}  (write  7  $col refclock)
        {$col == 7}  (write  9  $col refclock)
        {$col == 8}  (write  6  $col name)
        {$col == 9}  (write  8  $col name)
        {$col == 10} (write 10  $col cts_tpd)
      )
     )
    )
   )
  )
 )
)

(_default_script_
 (each (foreach col (if (defined $col) (write $col $col default_center) (write_force $col "-" undefined))))
)
