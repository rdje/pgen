#! /apps/perl/5.8.3/bin/perl -w -Iperl

use LinkedSpec;
#require HUtils;

#my $spec = qx(cat $ARGV[0]);
#my $data = q(bg:white textvariable:"\$widget_path::sdfdsf::efsdf" selectbackground:{\&black::dfndfldsfn:asdfdsfn::"} selectforeground:"white:");
#my $spec = qx(cat linkedspecs/easytk-opt.spec);
#my $data = qx(cat staflow_config.txt);
#my $data = qx(cat pplugin.txt);
#my $data = qx(cat conf/violators.conf);
#my $spec = qx(cat linkedspecs/simenv.spec);
#my $data = qx(cat plugin/genericfilter.plg);
#my $data = 'startpoint =~ /\//';
#my $data = qx(cat conf/tcfix.tk);
#my $data = qx(cat test.txt);
#my $data = qx(cat operators_try.txt);
#my $data = qx(cat vhdl.txt);
my $spec = qx(cat specs/pplugin.spec);
#my $spec = qx(cat specs/tablegrep.spec);
#my $spec = qx(cat specs/ifelse.spec);
#my $spec = qx(cat specs/tkgui.spec);
#my $spec = qx(cat specs/Lispish.spec);
#my $spec = qx(cat specs/hlink_substitution.spec);
#my $spec = qx(cat specs/vhdl.spec);
#
#my $anonymous = LinkedSpec::Get(\$spec);
#$anonymous->(\$data);
#while($anonymous->(\$data)) {}
#print '<'.join('> <', LinkedSpec::Get(\$spec)->(\$data)).">\n";

#$pplugin->{really_cool}->(1, 2, 3, 4, 5, 'Yeeeeeeeeeaaaaaaaaaaaaaaahhhhhhhhhhhhhhhhh')
#HUtils::Recurse(HUtils::Link("conf/qcing.conf"), sub {
# my ($info, $data) = @_;
#
#  if (ref $data) {
#   print "(@$info)<@$data>\n";
#  } else {
#   print "(@$info)[$data]\n";
#  }
#})

#print "Loading DATA..\n";
#my $data = do {local(@ARGV, $/) = "/home/sayinala/MPUSS_N3G2/CML_0.5_Release/Rev2_Mar28/Release/cells/sc_mpussn3g2v10gs60/cad_models/synopsys/sc_mpussn3g2v10gs60_w_125_1.08.lib"; <>};
#my $data = do {local(@ARGV, $/) = "/home/qdjeric/project/fsm/top_src/tstr_emp_mipi_csi2.vhd"; <>};
#my $data = do {local(@ARGV, $/) = "/home/qdjeric/project/fsm/afx/vfx/src/vspm_cmds_pkg.vhd"; <>};
#print "Loading SPEC..\n";
#my $spec = qx(cat specs/lib_reader.spec);
my $anonymous = LinkedSpec::Get(\$spec, pm_drive=>1);
#print "Appying SPEC to DATA..\n";
#$anonymous->(\$data);
