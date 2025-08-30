(lstype_long_i2chs
 (each (caseval {$ROW == 0} (foreach col (write $col $col header_wob))

            (if (defined 8)
               (casematch 8 
	       "DEV_(?:DES|VERIF)_IP_I2CHS" (foreach col (if (defined $col) (write $col $col black_on_beige)         (write_force $col - black_on_beige))) 
	       "DEV_IP_I2CHS"               (foreach col (if (defined $col) (write $col $col black_light_orange1)    (write_force $col - black_light_orange1))) 
	       "DEV_8500V1_IP_I2CHS"        (foreach col (if (defined $col) (write $col $col black_on_yellow)        (write_force $col - black_on_yellow))) 
	       "DES_[A-Z]\w+_IP_I2CHS"      (foreach col (if (defined $col) (write $col $col black_on_greenyellow_c) (write_force $col - black_on_greenyellow_c))) 
	       "REFERENCE_I2CHS"            (foreach col (if (defined $col) (write $col $col litepurple )            (write_force $col - litepurple))) 
	       "_I2CHS_"                    (foreach col (if (defined $col) (write $col $col black_on_lightblue_c)   (write_force $col - black_on_lightblue_c))) 
	           		            (foreach col (if (defined $col) (write $col $col default)                (write_force $col - default)))
	       )

	       (foreach col (if (defined $col) (write $col $col default)    (write_force $col - default)))
	     )

       )
 )
)
