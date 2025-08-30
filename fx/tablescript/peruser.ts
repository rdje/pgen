(path_type_labeled_consolidated_rep
 (insert 1 "Timing Path" StartPoint "StartPoint Clock" "SPC Edge" EndPoint "EndPoint Clock" "EPC Edge" "Slack (ns)")
 (each (caseval 
	 {$ROW == 0} (write 0 0 (concat (col 0) _ path_type))
	 {$ROW == 1} (foreach col (write $col $col header_wob))
                     (call rpt_consolidate_common)
  )
 )
)
