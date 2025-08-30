#! perl -w -I/home/djer/fx/fsm/afx/fx/perl

#=======================================================================
# Copyright (c) 2005-2007 Richard DJE. All rights reserved.
#
# This Perl CGI script is free software, you may redistribute it and/or 
# modify it under the same terms as Perl itself.
#=======================================================================

use PTiming;
use AmbiTiming;
use MagmaTiming;
use EncounTiming;
use HUtils;

use Digest::MD5;

use CGI;

my $cgi    = new CGI;

print "content-type: text/html\n\n";

my $info   = HUtils::Read(PathSearch->new->go('cgi'));
my %qs     = $cgi->Vars;
my $id     = Digest::MD5::md5_hex(join ("&", map {$_.'='.(defined $qs{$_} ? $qs{$_} :  "")} @{$$info{gp_id_key_list}}) , $$info{sepc}, Global->md5_encode);

if (defined $qs{id} && $id eq $qs{id} && -f $qs{file} && -s _ && -r _) {

 my @args    = ($qs{file}, index=>$qs{path_index}, noverbose=>1, split=>$qs{split});
 my $parser  = $qs{engine} eq 'pt'    ? PTiming->new     : 
              ($qs{engine} eq 'magma' ? MagmaTiming->new : 
              ($qs{engine} eq 'ambit' ? AmbiTiming->new  : 
		                        EncounTiming->new));
 my $content = $parser->read(@args);
 my @content = split /\n/, $content;
 
 $qs{se_nu}  = 1 unless defined $qs{se_nu};
 
 if (defined $qs{hl}) {
  for (split /,/o, $qs{hl}) {
   my ($min, $max, $bg, $fg) = do {
 	                       if (/(\d+)-(\d+)(?::(\w+)(?:-(\w+))?)?/o) {
 				       ($1, $2, $3, $4)
                                } elsif (/(-?\d+)(?::(\w+)(?:-(\w+))?)?/o) {
 				       (undef, $1, $2, $3)
                                }
          };
 
   $min ||= $max;
   $bg  ||= "yellow";
   $fg  ||= "black";
 
  $content[$_] =  qq/<font style="{background:$bg; color:$fg}">/.$content[$_]."</font>" foreach $min .. $max;
  }
 }
 
 $content[$_] =  qq/<span id="$_" onmouseover="span_onmouseover(this)" onmouseout="span_onmouseout(this)">/.$content[$_]."</span>" foreach 0 .. $#content;
 
 if ($qs{se_nu}) {
  my $i = -1;
  @content = map {$i++; sprintf("[%-3i]", $i)."  $_"} @content;
 }
 
 $content                  = join "\n", @content;
 
 print "<head>", 
        qq{<base href="http://$ENV{HTTP_HOST}">},
        qq{<script type="text/javascript" src="/cgi-bin/getpath.js">},
        "</script>",
       "</head>",
       "<pre>", 
         $content, 
       "</pre>";

}
