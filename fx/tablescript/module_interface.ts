(module_if_change
 (each (foreach col (write $col $col (if {$ROW == 0} (if {$col == 0} default_bold header_wob)
                                                     (casematch $col INPUT:          input
                                                                     OUTPUT:         output
                                                                     INOUT:          inout
             	                                        	     UNKNOWN:        white_on_dblue
             	                                        	     INPUT_WARNING:  red_on_yellow
             	                                        	     OUTPUT_WARNING: yellow_on_red
             	                                        	     INOUT_WARNING:  yellow_on_brown
             	                                        	     DIFFERENT:      black_light_orange
             	                                        	     -               undefined
             	                                        	                     default_center
             	                                        	                

                                                     )
         
      )
    )
  )
 )
)

// (module_if
//  (unshift Port Direction Size MSI LSI File)
//  (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
//         (else
//          (write 0 0 portname)
//          (write 1 1 (casematch 1 IN input OUT output inout))
//          (if {@(2) =~ /VECTOR/o} (then
//            (write_force 2 (eval {@(3) - @(4) + 1}) number_nf)
//            (write  3 3 number_nf)
//            (write  4 4 number_nf)
// 
//           ) (else
//            (write_force 2 1 number_nf)
//            (write_force  3 - undefined)
//            (write_force  4 - undefined)
//           )
//          )
// 
//          (if (defined 5) (write 5 5 file_path) (write_force 5 - undefined))
//         )
//    )
//  )
// )

(module_if
 (unshift Port Direction Size MSI LSI File)
 (each (if {$ROW == 0} (foreach col (write $col $col header_wob))
        (else
         (write 0 0 portname)
         (write 1 1 (casematch 1 IN input OUT output inout))
         (if {@(2) =~ /VECTOR/o} (then
           (write_force 2 (eval {@(3) - @(4) + 1}) number_nf)
           (write  3 3 number_nf)
           (write  4 4 number_nf)

          ) (else
           (write_force 2 1 number_nf)
           (write_force  3 - undefined)
           (write_force  4 - undefined)
          )
         )

         (if (defined 5) (write 5 5 file_path) (write_force 5 - undefined))
        )
   )
 )
)

(module_link
 (unshift "Block List")
 (each (write 0 0 (if {$ROW == 0} header_wob black_on_lgreen_bold)))
)

(labeled_vhdl_entities
 (insert 1 Port Direction Size Type MSI LSI)
 (each (caseval 
	 {$ROW == 0} (foreach col (if {$col == 0} 
				    (write 0 (http (subst {^ent:} {} (col 1)) (col 0)) black_on_greenyellow_b_s11)
				    (unless (match $col ^ent:)
				      (if (match $col ^archi:) (write (expr  $col -1) (http (subst {^\w+:|:\w+$} {} (col $col) (global 1)) (subst {^\S+:} {} (col $col))) white_on_blue_bold)) 
				    )
				  )
	             )

	 {$ROW == 1} (foreach col (write $col $col header_wob))
                     (do
                      (write 0 0 black_on_beige)
                      (write 1 1 (casematch 1 "(?i)IN\b" input "(?i)\bOUT" output inout))
		      (write 2 2 (casematch 2 -   undefined black_on_moccasin_c))
		      (write 3 3 (casematch 3 -   undefined "(?i)std_logic(?:_vector)?" black_on_lavender_c black_on_yellow_wb))
		      (write 4 4 (casematch 4 ^-$ undefined ^\d+$ number_nf black_on_lightblue_c))
		      (write 5 5 (casematch 5 -   undefined number_nf))
                     )
  )
 )
)
