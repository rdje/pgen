(filterout_ts
 (unshift  Mode IOMode Corner "File Path" StartPoint "StartPoint Clock" "SPC Edge" EndPoint "EndPoint Clock" "EPC Edge" "Slack (ns)")
 (each (if {$ROW == 0}
   (foreach col (write $col $col header_wob))
   (else
    (call rpt_consolidate_common)
    (store launch_start_time)
    (store capture_start_time)
    (store launch_clock_path_delta)
    (store capture_clock_path_delta)
   )
  )
 )
)
