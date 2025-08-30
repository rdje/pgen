#===================================================================
# Copyright (c) 2005-2007 Richard DJE. All rights reserved.
#
# This Perl module is free software, you may redistribute it and/or 
# modify it under the same terms as Perl itself.
#===================================================================
package XLSreader;

use Spreadsheet::ParseExcel;

sub openwb {
my ($excelfile, $quiet)=@_;

 print "(xlsreader) -I- Loading Excel file '$excelfile'..\n" unless $quiet;

 my $wb = Spreadsheet::ParseExcel::Workbook->Parse($excelfile);

 # The *defined* is REQUIRED !!
 unless(defined $wb) {
  print "(xlsreader) -E- Can't open Excel file '$excelfile'.\n";
  return undef
 }

 return $wb
}


sub getws {
my ($wb, $sheetspec)=@_;

 my $ws = $wb->Worksheet($sheetspec); 

 # The *defined* is REQUIRED !!
 unless(defined($ws)) {
  print "(xlsreader) -E- Can't get sheet '$sheetspec'\n";
  return undef
 }
 
 return $ws
}


sub get { 
my ($ws, $srow, $scolumn, $width, $height)=@_;
my @rowdata;

 my $row=0;
 while ($row < $height) {
  my @coldata=();
  foreach my $col (0 .. $width-1) {
   my $cell=$ws->Cell($srow+$row, $scolumn+$col);
   push @coldata, (defined($cell->{Val}) && length($cell->{Val}) ? $cell->{Val} : undef);
  }

  push @rowdata, [@coldata];

  $row++
 }

 return [@rowdata]
}


1
