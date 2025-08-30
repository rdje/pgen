#===================================================================
# Copyright (c) 2005-2007 Richard DJE. All rights reserved.
#
# This Perl module is free software, you may redistribute it and/or 
# modify it under the same terms as Perl itself.
#===================================================================
package Table2SS;

use Spreadsheet::WriteExcel::Big;
use Spreadsheet::WriteExcel::Utility;

use HUtils;
use TableScript;

# Loading the main configuration file
my $conf        = HUtils::Link(PathSearch->go(Global->table2ss_conf));

# Retrieving the index map
   $conf->{script_index_info} = TableGrep::Map();

sub UConf {my ($userconfile) = @_; HUtils::Merge($conf, ref($userconfile) ? $userconfile : HUtils::Link($userconfile))}

# The output of RCAllocate ** should ** be rendered in a single sheet
sub RCAllocate {
my ($baserow, $basecol, $table_per_row, $tlinfo, $script) = splice @_, 0, 5;

 my %option = @_;

 # If X padding and Y padding are not defined or zero then use the default
 $option{xpad} = defined($option{xpad}) && $option{xpad} > 0 ? $option{xpad} : $conf->{_rcallocate}{xpad};
 $option{ypad} = defined($option{ypad}) && $option{ypad} > 0 ? $option{ypad} : $conf->{_rcallocate}{ypad};

 print "(Table2SS::RCAllocate) -I- Building a 2D-array of table, containing $table_per_row table per row..\n";

 # Building a 2D-array of table, containing $table_per_row table per row
 my @a2d   = @{List2Table($script ? TableListHandler($tlinfo, $script, %option) : $tlinfo, $table_per_row)};
 my $a2dsz = @a2d;

 my @column_width;
 my @tables_widths;
 # Finding the widest table belonging to a given column
 for (my $c=0;  $c < $table_per_row; ++$c) {
  # Determining the max width
  my $maxwidth = 0;
  for (my $r=0;  $r < $a2dsz;         ++$r) {
   # This will only happen on the last row
   # Table $r/$c must exist.
   last unless defined $a2d[$r][$c];
  
   # If the $r/$c Table is empty, set its width to zero.
   $tables_widths[$r][$c] = @{$a2d[$r][$c]} ? (sort {$b <=> $a}  map {scalar(grep {ref} @$_)}  @{$a2d[$r][$c]})[0] : 0;
   $maxwidth = $tables_widths[$r][$c] if $maxwidth < $tables_widths[$r][$c]
  }

  $column_width[$c] = $maxwidth;
 }
 
 print "(Table2SS::RCAllocate) -I- Allocating Row-Column pairs..\n";

 my @otlist;
 my @na2d;
 # Allocating Row-Column pairs
 for (my $r=0;  $r < $a2dsz;         ++$r) {
 for (my $c=0;  $c < $table_per_row; ++$c) {
  # This will only happen on the last row
  last unless defined $a2d[$r][$c];

  $na2d[$r][$c] = do {
	   if ($r == 0 && $c == 0) {
              {
               row    => $baserow                                      ,
               col    => $basecol                                      ,
	       a1     => xl_rowcol_to_cell($baserow, $basecol)         ,
               width  => $tables_widths[$r][$c]                        ,
               heigth => scalar(@{$a2d[$r][$c]})                       ,
               data   => $a2d[$r][$c]
              }

            } elsif ($r == 0 && $c >  0) {
              my $col = $na2d[$r][$c-1]{col} + $column_width[$c-1] + $option{xpad};

              {
               row    => $baserow                                       ,
               col    => $col                                           ,
	       a1     => xl_rowcol_to_cell($baserow, $col)              ,
               width  => $tables_widths[$r][$c]                         ,
               heigth => scalar(@{$a2d[$r][$c]})                        ,
               data   => $a2d[$r][$c]
              }

            } elsif ($r >  0 && $c == 0) {
              my $row = $na2d[$r-1][$c]{row} + $na2d[$r-1][$c]{heigth} + $option{ypad};
              {
               row    => $row                                            ,
               col    => $basecol                                        ,
	       a1     => xl_rowcol_to_cell($row, $basecol)               ,
               width  => $tables_widths[$r][$c]                          ,
               heigth => scalar(@{$a2d[$r][$c]})                         ,
               data   => $a2d[$r][$c]
              }

            } else {
              my $row = $na2d[$r-1][$c]{row} + $na2d[$r-1][$c]{heigth} + $option{ypad};
              my $col = $na2d[$r][$c-1]{col} + $column_width[$c-1]     + $option{xpad};

              {
               row    => $row                                            ,
               col    => $col                                            ,
	       a1     => xl_rowcol_to_cell($row, $col)                   ,
               width  => $tables_widths[$r][$c]                          ,
               heigth => scalar(@{$a2d[$r][$c]})                         ,
               data   => $a2d[$r][$c]
              }
            }
   };

   push @otlist, $na2d[$r][$c]
  }
 }
 
 return \@otlist
}

sub SpliTable {my ($table, $divide_by) = @_; List2Table($table, int(@$table/$divide_by))}

sub SwapTable {
my ($table) = splice @_, 0, 1;

 my %option = @_;

 if ($option{padding}) {
  # In case explicit padding is requested, then we should
  # check for the size of the rows
  my $maxsize = 0;
  my %sizecnt = map {my $size = @$_; $maxsize = $size if $maxsize < $size; $size => 1} @$table;
  foreach (@$table) {
   my $diff = $maxsize - @$_;
   push @$_, (' ') x $diff if $diff
  }
 }

 my @swapped;
 while(@$table) {
  my $last = pop @$table;
  my $i    = 0;
  push @{$swapped[$i++]}, $_ foreach (@$last)
 }

 \@swapped
}

sub PruneDummyColumn {
my ($table) = @_;

 # All Rows should be of the same width
 my %widthcnt = map {scalar(@$_) => 1} @$table;
 my @widthcnt = keys %widthcnt;

 die "(Table2SS::PruneDummyColumn) -E- Rows don't have the same size," if @widthcnt > 1;
 my $size = $widthcnt[0];
 my %prune_colist;
 foreach my $ccol (0 .. ($size-1)) {
  my %colv = map {$$_[$ccol] => 1} @$table;
  my @colv = keys %colv;
  
  $prune_colist{$ccol} = 1 if @colv == 1;
 }

 return [map {my $coli=0; [grep {!$prune_colist{$coli++}} @$_]} @$table]
}

sub prune_column {
my ($table, $colist) = @_;

 my %colist = map {$_ => 1} @$colist;

 return [map {my $coli=0; [grep {!$colist{$coli++}} @$_]} @$table]
}

sub list_prune_column {
my ($list, $colist) = @_;

 my %colist = map {$_ => 1} @$colist;

 my $coli=0;
 return [grep {!$colist{$coli++}} @$list]
}

sub DriveSheet {
my ($wbid, $sheetname, $rcallocate) = @_;

 print "(Table2SS::DriveSheet) -I- Driving sheet *$sheetname*..\n";

 # Sheetname length should be <= 31, that's an MS Excel constraint 
 unless (length ($sheetname) <= 31) {
  print "(Table2SS::DriveSheet) -W- Sheetname is too long, fixing it.\n";
  $sheetname = join '', (split //, $sheetname)[0 .. 30];
 }

 if (defined $conf->{workbook}{$wbid}{sheets}{$sheetname}) {
  print "(Table2SS::DriveSheet) -W- WorkBook *$conf->{workbook}{$wbid}{name}* already contain sheet *$sheetname*, overriding it.\n";
 } elsif (defined $conf->{workbook}{$wbid}{handle}) {
  $conf->{workbook}{$wbid}{sheets}{$sheetname} = $conf->{workbook}{$wbid}{handle}->add_worksheet($sheetname);
 } else {
  print "(Table2SS::DriveSheet) -W- Wrong WorkBook ID *$wbid*.\n";
  exit 3
 }

 DriveTable($conf->{workbook}{$wbid}{sheets}{$sheetname}, $_) foreach (@{ref($rcallocate) eq 'ARRAY' ? $rcallocate : $rcallocate->{rca}});
 $conf->{workbook}{$wbid}{sheets}{$sheetname}->set_zoom(75);

 # Returning the sheet reference
 $conf->{workbook}{$wbid}{sheets}{$sheetname}
}

sub GetSheet {
my ($wbid, $sheetname) = @_;

 if (defined $conf->{workbook}{$wbid}{sheets}{$sheetname}) {
  $conf->{workbook}{$wbid}{sheets}{$sheetname}
  } else {
  print "(Table2SS::GetSheet) -W- Can't retrieve sheet '$sheetname'.\n";
  exit 3
 }
}

sub DriveTable {
my ($sheet, $table) = @_;

 foreach my $r  (0 .. $table->{heigth}-1) {
  foreach my $c (0 .. $table->{width}-1)  {

   # Will call the write command only if the current cell is defined
   next unless defined $table->{data}[$r][$c];

   unless (defined $table->{data}[$r][$c]{url}) {
	   #print "[$r][$c] ($table->{data}[$r][$c]{value})\n";
    $sheet->write(
	              $table->{row} + $r             ,
	              $table->{col} + $c             ,
		      $table->{data}[$r][$c]{value}  ,
		      $conf->{format_info}{$table->{data}[$r][$c]{format}}
	         )
   } else {
    $sheet->write_url(
	              $table->{row} + $r             ,
	              $table->{col} + $c             ,
		      $table->{data}[$r][$c]{url}    ,
		      $table->{data}[$r][$c]{value}  ,
		      $conf->{format_info}{$table->{data}[$r][$c]{format}}
	             )
   }
  }
 }
}


sub AddWorkBook {
my ($workbook_name) = @_;

 unless (defined $conf->{workbooks}{$workbook_name}) {
  my $wbid       = scalar(keys %{$conf->{workbook}});

  my $cworkbook  = Spreadsheet::WriteExcel::Big->new($workbook_name); 
  unless (defined $cworkbook) {
   print "(Table2SS::AddWorkBook) -E- Problem creating Workbook *$workbook_name*.\n";
   exit 2
  }

  # Should avoid doing this for each new workbook
  $conf->{format_info}             = add_format($cworkbook, $conf);
  
  $conf->{workbook}{$wbid}{handle}   = $cworkbook;
  $conf->{workbooks}{$workbook_name} = $wbid;

 } else {
  return $conf->{workbooks}{$workbook_name}
 }
}

sub Close            {$conf->{workbook}{$_[0]}{handle}->close()}
sub TableListHandler {my ($tlinfo, $script) = splice @_, 0, 2; my %option = @_; my $idx = -1; [map {++$idx; TableScript::Run($conf, $_, $option{table2script} ? &{$option{table2script}}(table_index=>$idx) : $script)} @$tlinfo]}
sub TableHandler     {my ($tlinfo, $script) = @_; TableScript::Run($conf, $tlinfo, $script)}

sub List2Table {
my ($list, $width) = @_;

 my @table;
 my @ta;
 my $index = 0;
 foreach (@$list) {
  if ($index && ($index % $width == 0)) {
   push @table, [@ta];
   @ta = ();
  }
  
  push @ta, $_;
  $index++; 
 }

 push @table, [@ta] if @ta;

 return [@table]
}


sub TableTreeAllocate {
my $tabletree = shift;

 my %option = @_;
 HUtils::Recurse($conf->{_tabletreeallocate}, sub {$option{$_[0][0]} ||= $_[1]}); 

 my %link;
 my $index = 0;
 my @tlist;
 my @table2script;
 HUtils::Recurse($tabletree, sub {my ($info, $table) = @_;
  eval q/$link{qq(/.join(q/)}{qq(/, @$info).q/)} = $index/;
  push @tlist, $table;
  push @table2script, [@$info] if $option{table2script};
  ++$index;
 });

 {rca=> RCAllocate($option{baserow}, 
		   $option{basecol}, 
		   $option{table_per_row}, 
		   [@tlist], 
		   $option{script}, 
		   xpad=>$option{xpad}, 
		   ypad=>$option{ypad},
                   table2script=> $option{table2script} ? sub {&{$option{table2script}}(@_, key2table=>\@table2script)} : undef
	          ), 

  'link'=>\%link
 }
}

sub TableTreeA1 {
my ($tta, $keylist) = @_; 

 my $pos = eval q/$tta->{link}{qq(/.join(q/)}{qq(/, @$keylist).q/)}/;

 $tta->{rca}[$pos]{a1}
}

sub TableTreeLeaf {
my ($tta, $keylist) = @_; 

 my $pos = eval q/$tta->{link}{qq(/.join(q/)}{qq(/, @$keylist).q/)}/;

 $tta->{rca}[$pos]{data}
}

sub DriveTableTree {
my ($wbname, $sheet, $tabletree) = splice @_, 0, 3;

 my %option = @_;
 HUtils::Recurse($conf->{_drivetabletree}, sub {$option{$_[0][0]} ||= $_[1]}); 

 my $tta = TableTreeAllocate($tabletree, map {$_ => $option{$_}} grep !/link_/oi, keys %option);

 my $linkcode = $option{link_code};
 my @link_list;
 $option{table_sheet} ||= "${sheet}_tables";
 HUtils::Recurse($tta->{'link'}, sub {my ($info, $index) = @_;
  my $n_info = $linkcode && ref($linkcode) eq 'CODE' ? $linkcode->($info, TableTreeLeaf($tta, $info)) : $info;
  push @link_list, [map {"internal:$option{table_sheet}!$tta->{rca}[$index]{a1}\@$_"} @$n_info];
 });
 
 my $rca_link_list = RCAllocate($option{link_baserow}, $option{link_basecol}, 1, [[@link_list]], $option{link_script});
 
 my $wbid          = AddWorkBook($wbname);

 DriveSheet($wbid, $sheet,               $rca_link_list);
 DriveSheet($wbid, $option{table_sheet}, $tta);
 Close($wbid) unless $option{keep_opened};
}


#------------ add_format ----------------
sub add_format {
my ($wb, $config) = @_;
my %ret_formats;

my $colors = &set_custom_color($wb, $config->{custom_color});

 foreach (keys %{$config->{format}}) {
  my %fm_hash = %{$config->{format}{$_}};
  $fm_hash{bg_color} = $colors->{$fm_hash{bg_color}} if exists $fm_hash{bg_color};
  $ret_formats{$_}   = $wb->add_format(%fm_hash);
 }

 # Defining a new $config entry for later on-the-fly format creation
 # due to diff'ing algorythm
 $config->{CUSTOM_COLORS} = {%$colors};

 return {%ret_formats}
}

#-------- set_custom_color --------------
sub set_custom_color {#  $custom_color corresponds to $config->{custom_color}
my ($wb, $custom_color) = @_;
my %colors;

my $color_index=15;

 $colors{$_} = $wb->set_custom_color($color_index++, @{$custom_color->{$_}}) foreach (keys %$custom_color);

 return {%colors}  
}

1;
