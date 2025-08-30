#! perl -w -I/home/djer/fx/fsm/afx/fx/perl

#=======================================================================
# Copyright (c) 2007 Richard DJE. All rights reserved.
#
# This Perl CGI script is free software, you may redistribute it and/or 
# modify it under the same terms as Perl itself.
#=======================================================================

use HUtils;
use PPlugin;

use File::Spec;
use File::Temp;
use Digest::MD5;
use CGI;

my $cgi        = new CGI;
my $p          = new PPlugin;

my $info       = HUtils::Read(PathSearch->go('cgi'));

my $ext        = join '|', keys %{$$info{gf_enscript_extensions}};
my $ext_re     = qr/$ext$/io;
my $enscript   = Global->enscript;

my %qs         = $cgi->Vars;

my $id         = Digest::MD5::md5_hex((map {$_.'='.(defined $qs{$_} ? $qs{$_} :  "")} grep {!/\./} @{$$info{gf_id_key_list}}) , $$info{sepc}, Global->md5_encode);


if (defined $qs{id} && $id eq $qs{id} && -f $qs{file} && -r _) {
 my $filename   = (File::Spec->splitpath($qs{file}))[2];

 if ($qs{file} =~ $ext_re) {
  open (my $f, "$enscript --line-numbers -E$$info{lang}{$+} --color --language=html -q -o - $qs{file} |"); local $/; my $file = <$f>;
  $file =~ s#<h1>.+?</h1>##io;
  $file =~ s#(?<=<title>).+?(?=</title>)#$qs{file}#io;
  $file =~ s#<address>.+?</address>##io;
 
  print  $file;
 
 } elsif ($qs{file} =~ /\.pdf$/oi) {
  print "content-type: application/pdf\n";
  print "content-disposition: attachment; filename=$filename\n\n";
  open (my $f, $qs{file}); local $/; my $file = <$f>;
 
  print $file;

 } elsif ($qs{file} =~ /\.xls$/oi) {
  print "content-type: application/vnd.ms-excel\n";
  print "content-disposition: attachment; filename=$filename\n\n";
  open (my $f, $qs{file}); local $/; my $file = <$f>;
 
  print $file;
 
 } elsif ($qs{file} =~ /\.ppt$/oi) {
  print "content-type: application/vnd.ms-powerpoint\n";
  print "content-disposition: attachment; filename=$filename\n\n";
  open (my $f, $qs{file}); local $/; my $file = <$f>;
 
  print $file;
 
 } elsif ($qs{file} =~ /\.html?$/oi) {
  open (my $f, $qs{file}); local $/; my $file = <$f>;
  print $file
 
 } elsif ($qs{file} =~ /\.(?:flt|clt)$/oi) {
  open (my $f, $qs{file}); local $/; my $file = <$f>;


  Global->set ('cgi')           = $info;
  Global->set ('http_hostport') = $ENV{SERVER_NAME};

  my $fo  = $p->exec('file_list_path2http', $file);
  print "content-type: text/html\n\n";
  print "<html><head><title>$qs{file}</title></head>";
  print "<body><pre>$fo</pre><hr></body>";
  print "</html>";
 
 } elsif (-T $qs{file}) {
  open (my $f, $qs{file}); local $/; my $file = <$f>;

  print "content-type: text/html\n\n";
  print "<html><head><title>$qs{file}</title></head>";
  print "<body><pre>$file</pre><hr></body>";
  print "</html>";
 
 } else {
  print "content-type: application/octet-stream\n";
  print "content-disposition: attachment; filename=$filename\n\n";
  open (my $f, $qs{file}); local $/; my $file = <$f>;
 
  print $file;
 
 }
}
