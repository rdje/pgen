#! perl -w -I/home/djer/fx/fsm/afx/fx/perl

#=======================================================================
# Copyright (c) 2007 Richard DJE. All rights reserved.
#
# This Perl CGI script is free software, you may redistribute it and/or 
# modify it under the same terms as Perl itself.
#=======================================================================

use HUtils;

use Digest::MD5;


my $info       = HUtils::Read(PathSearch->new->go('cgi'));
