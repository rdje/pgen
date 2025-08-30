#------------ class ------------------

sub class {
 return &hash_of_hash(shift @_)
}

sub table_order {
return &simple_array(shift @_)
}

sub tablecss {
return &simple_hash(shift @_)
}

sub html_estylesheet {
my ($config) = @_;
  
 print "<head>\n";
 print "<title> Hello world !</title>\n";
 print "<style type=\"text/css\">\n";
 foreach my $class (keys %{$$config{class}}) {
  print ".$class {";
  my $onebefore=0;
  foreach my $property (keys %{$$config{class}{$class}}) {
   print " ;" if $onebefore;
   print "$property: ";
   my $pv = $$config{class}{$class}{$property};
   if (ref $pv) {
    print join(" ", map {$$config{custom_colors}{$_} && sprintf("#%02x%02x%02x", @{$$config{custom_colors}{$_}}) || $_} @$pv), "\n";
   } else  {
     print $$config{custom_colors}{$pv} && sprintf("#%02x%02x%02x", @{$$config{custom_colors}{$pv}})|| $pv, "\n";
   }

   ++$onebefore 
  }
  print "}\n";
 }
 print "</style>\n";
 print "</head>\n";
}

sub html_tablerow {
my ($info, $curow, $mask, $config, $section) = @_;

my @rowdata;

 $$config{COLUMN_NUM} = 0;

 push @rowdata,"<tr>";
 # If the current LOF file section is mapped with either a script having its name (default mapping) or 
 # explicitly mapped with a script using the *scriptmap* CONF section then..
 if (exists $config->{scripts}{$section} || exists $config->{scriptmap}{$section}) {
  my $scriptname;

  $scriptname = $section   		       if exists $config->{scripts}{$section};
  $scriptname = $config->{scriptmap}{$section} if exists $config->{scriptmap}{$section};

  #=======================================================
  # Important: Even though references spring into existence
  # when used in situation assuming their existence,
  # the following hash ref initialization is mandatory 
  # for the rendering to work even on the very first 
  # outputed line.
  #
  # Please don't ask me why :)
  #=======================================================
  my $lineout = {};
  if (exists $$mask[0] && ref($$mask[0]) && exists $config->{checkinfo}{diff}{ref($$mask[0]) eq 'ARRAY' ? 'added' : 'removed'}{$section}) {
   # If we are in DIFF mode, and the current line needs to Added/Removed, and if the current LOF section is associated with
   # a script in DIFF mode, then call that script to retrieve the formatting for the current line.
   my $diffscript = $config->{checkinfo}{diff}{ref($$mask[0]) eq 'ARRAY' ? 'added' : 'removed'}{$section};
   unless (exists $config->{scripts}{$diffscript}) {
    print "(a2xhs) -E- Diff-mode script '$diffscript' of section '$section' is not defined in the configuration file.\n";
    exit 1
   }

   &script_start($config, $lineout, $curow, $info, $config->{scripts}{$diffscript});
  } else {
   &script_start($config, $lineout, $curow, $info, $config->{scripts}{$scriptname});
  }


  foreach my $myent (sort keys %$lineout) {
   my $from = $$lineout{$myent}{from};

   $$config{COLUMN_NUM} = $myent if $$config{COLUMN_NUM} < $myent;

   my $format;
   if (defined($from) && exists($$mask[$from]) && !ref($$mask[$from])) { # Write command
    $format = $$mask[$from]
   } else {
    $format = $$lineout{$myent}{format}
   }

   #print "<td class=$format title=\"\">$$lineout{$myent}{value}</td>\n";
   push @rowdata, "<td class=$format >$$lineout{$myent}{value}</td>";
  }
  
  $$config{COLUMN_NUM} += 1;
 } else {
   $$config{COLUMN_NUM} = @$info if $$config{COLUMN_NUM} < @$info;

   foreach (0 .. $#$info) {
    my $format = exists $$mask[$_] ? $$mask[$_] : "";
    push @rowdata, "<td class=$format title=\"\">$$info[$_]</td>";
   }
 }

 push @rowdata, "</tr>";

 return join("\n", @rowdata)
}

sub htmlcss_driver {
my  ($masks,
     $inputdata, 
     $outputfile, 
     $config,
     $default_sheetname) = @_;


 print "(a2xhs) -I- Driving HTML file '$outputfile'..\n";

 my @allsections = keys %$inputdata;
 unless(@allsections) {
  print "(a2xhs) -W- No data to process.\n";
  return
 }

 print "<html>\n";
 
 &html_estylesheet($config);

 print "<body>\n";
 foreach my $section (@{$$config{table_order}}) {
  print "(a2xhs) -I- Writing '$section' table..\n";

  print "<table cellspacing=1";
  if (exists $$config{tablecss}{$section}) {
   print " class=$$config{tablecss}{$section}>\n"
  } else {
   print ">\n"
  }

  my $row=0;
  foreach my $irow (@{$inputdata->{$section}}) {
    print  &html_tablerow($irow, 
		  	  $row, 
		  	  $$masks{$section}[$row], 
		  	  $config, 
		  	  $section), "\n";

    $row++
  }

  print "</table>\n\n";
 }

 print "</body>\n";
 print "</html>\n";


 print "(a2xhs) -I- Done.\n";
}

1
