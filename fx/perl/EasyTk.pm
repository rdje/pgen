#===================================================================
# Copyright (c) 2005-2007 Richard DJE. All rights reserved.
#
# This Perl module is free software, you may redistribute it and/or 
# modify it under the same terms as Perl itself.
#===================================================================
package EasyTk;

use Tkx;
use TkGui;
use PathSearch;

sub Init {
my ($guifile, $topwidget) = splice @_, 0, 2;

 my %optional = @_;

 my %crail;
 $crail{config} = tkconfread();

 Tkx::lappend("auto_path", "/home/qdjeric/install/lib/Tix8.4.2");
 Tkx::package_require("Tix");

 my $gui_data   = TkGui::Get($guifile);

 if (exists $gui_data->{$topwidget}) {
  unshift @{$crail{gui}}, {data=>$gui_data, file=>$guifile};

  Walk(\%crail, $crail{gui}[0]{data}{$topwidget});

  #shift @{$crail{gui}};
  
  &{$optional{startup}}(\%crail)             if exists $optional{startup};
  if (exists $optional{winfo} && ref($optional{winfo}) eq 'HASH') {
   %{$optional{winfo}} = %crail;

   #HUtils::Recurse($optional{winfo}, \&display);
  }
  
  
  Tkx::MainLoop();
 }

}

sub display {
my ($info, $value) = @_;

 print "display: @$info - <$value>\n";

}

sub tkconfread {HUtils::Link($_[0] || PathSearch->go('easytk'))}

sub Walk {
 unshift @{${$_[0]}{pack}}, 1;
 my $w = tkwalk(@_);
 shift @{${$_[0]}{pack}};
 event_attach($_[0]);

 return $w
}

sub tkwalk {
my ($crail, $widget_info, $parent) = @_;

 die "(tkwalk) -E- Input widget is undefined, " unless $widget_info;

 $parent = Tkx::widget->new($parent || ".");

 die unless $parent;
 
 my $leveltype= $$widget_info[0];
 # leveltype not corresponding to widgets don't need a name !


 my $lvname;
 unless ($leveltype =~ /^include$/o) {
  my $lobj;
  my $first_index;

  my %optdata = tkpreprocess($crail, $$widget_info[1]);
  unless ($leveltype =~ /$$crail{config}{skipre}/o) {
   $lvname = lcfirst($$widget_info[1][0]);

#   print "tkwalk($parent)($leveltype)($lvname)\n";

   push @{$$crail{namelifo}}, $lvname;

   my $widgetname = $$crail{config}{tix}{builtin}{$leveltype} || $$crail{config}{tix}{wa}{$leveltype} || $$crail{config}{tix}{$leveltype} || $leveltype;

   #print "$widgetname = $$crail{config}{names}{$leveltype} - $leveltype\n";

   #my $withoutcmd  = join '', map {" $_ {$optdata{$_}}"} grep {!/$$crail{config}{cmd_and_var_re}/o} keys %optdata;
   my @withoutcmd  = map {$_, $optdata{$_}} grep {!/$$crail{config}{cmd_and_var_re}/o} keys %optdata;

   print "($leveltype) <$widgetname>\t<$parent>.<$lvname>\t<@withoutcmd>\n";
   #$lobj = &{"Tkx::".$widgetname}($parent.$lvname, $withoutcmd);
   my $new_foo = "new_$widgetname";
   $lobj = $parent->$new_foo(-name=>$lvname, @withoutcmd);
   #$lobj = Tkx::widget->new("$parent.$lvname");

    

#   print q(\$$crail{winfo}{').join('.',  @{$$crail{namelifo}}).q('}).$lobj."\n";
   ${eval q(\$$crail{winfo}{').join('.',  @{$$crail{namelifo}}).q('})} = $lobj;

   $first_index = 1;
  } else {
   $first_index = 0;
  }
  
  my $last_index  = @{$widget_info->[1]} - 1;
  my $retv        = &{$$crail{config}{handler}{$leveltype} || $leveltype}($crail, [@{$$widget_info[1]}[$first_index .. $last_index]], $leveltype, $lvname, $lobj, $parent);  

  unless ($leveltype =~ /$$crail{config}{skipre}/o) {
   my @withcmd  = map {$_ => $optdata{$_}} grep { /$$crail{config}{cmd_and_var_re}/o} keys %optdata;
   @withcmd && $lobj->configure(@withcmd);
   
   pop @{$$crail{namelifo}};
  }

  return $retv
 } else {
  # Here $lvname contains the path value of the new file to be analyzed
  return tkanalyze($crail, $lvname, $parent)
 }
}


# Not sure if the following is still useful !??!
sub tkanalyze {
my ($crail, $tklisp, $parent) = @_;

 return Walk($crail, Lispish::single($tklisp), $parent);
}

sub tkpreprocess {
my ($crail, $args) = @_;

#print "\n############## ENTERING PREPROCESS crail($crail)############\n";
 my $trigger = $crail->{config}{preprocess}{trigger} || die "(EasyTk::tkpreprocess) -E- No 'trigger' parameter defined,";
 my $special = $crail->{config}{preprocess}{special} || die "(EasyTk::tkpreprocess) -E- No 'special' parameter defined,";

 my @options;
 my @preproc;
 foreach (@$args) {
  unless (ref($_) && $$_[0] =~ /$trigger/o) {
#	  print "NOT a reference <$_>\n" unless ref;
#	  print "NOT a trigger argument <$$_[0]>\n" if ref;
   push @preproc, $_;
   next
  }

#  print "SEEN *$$_[0]*\n";
  
  if ($$_[0] =~ /$special/o)  {
#	  print "SPECIAL\n";
   push @preproc, $_;
   push @options, get_opt() 
  } else {
#	  print "FCALL\n";
   my $cmd = shift @{$$_[1]};

   $cmd =~ /[a-zA-Z]\w*(?:::[a-zA-Z]\w*)?/ || die "(EasyTk::tkpreprocess) -E- Invalid function name '$cmd',";
   $cmd = "main::$cmd" unless $cmd =~ /\w+::\w+/o;
   
   unless (*$cmd{CODE}) {
    print "(EasyTk::tkpreprocess) -I- Unknown function '$cmd', Skipping.\n";
    next
   }

#   print "CALLING *$cmd*\n";

   push @preproc, &$cmd(@{$$_[1]})
  }
 }

 # Actual Update of the initial argument list
 @$args = @preproc;

 # Hamdling of options, if any
 my %options = @options;
 my %ret = map {-$_ => (/$$crail{config}{cmd_and_var_re}/o ? eval do {
				                                     $options{$_} =~ s/([\$\&])(?=[a-zA-Z])/$1main::/g; 
								     "$options{$_}"
			                                            } : $options{$_})} keys %options; 
# print "############## LEAVING PREPROCESS ############\n";
 return wantarray ? %ret : join('', map {" -$_ {$options{$_}}"} keys %options)
}


sub get_opt {return ref($_) ? map {(/(\S+?):(.+)/so)} @{$$_[1]} : ()}


sub tkgeneric {
my ($crail, $widget_args, $leveltype, $lvname, $lobj) = @_;

#print "tkgeneric: leveltype<$leveltype> lvname<$lvname> lobj<$lobj>\n";

 my $is_panedwindow = $leveltype =~ /panedwindow/io;
 my $is_menubutton  = $leveltype =~ /menubutton/io;
 my $is_menu        = $leveltype =~ /menu\b/io;
 my $is_toplevel    = $leveltype =~ /toplevel\b/io;
 
 my $pack=[];
 foreach (@$widget_args) {
  next if ref($_) && $$_[0] =~  /$$crail{config}{preprocess}{special}/o;
  #
  # Some entry might not be reference
  #
  if (ref($_) && exists $$crail{config}{handler}{$$_[0]} && !$$crail{config}{$leveltype}{$$_[0]}) {
   # Now the current widget is seen as the parent of the
   # current sub level widget
   unshift @{$crail->{pack}}, 1;
   my $slevel= tkwalk($crail, $_, $lobj);
   shift @{$crail->{pack}};

   next if $$crail{config}{skip}{$$_[0]};

   $is_panedwindow && $lobj->add($slevel->{obj});
   $is_menubutton  && $lobj->configure(-menu=>$slevel->{obj}, -text=>$slevel->{name});
   $is_menu        && $lobj->add('cascade',  -label=>$slevel->{name}, -menu=>$slevel->{obj});

  } else {
    # For dealing with the 'separator' command of 'menu' widgets
    my $type = ref($_) ? $$_[0] : $_;

    my %options  = get_opt();
    my @arguments= map {-$_ => (/$$crail{config}{cmd_and_var_re}/o ? eval do {
								 $options{$_} =~ s/([\$\&])(?=[a-zA-Z])/$1main::/g;
								 "$options{$_}"
								 } : $options{$_})} keys %options;
    
    if ($is_menu) {
     unshift @arguments, $type;
#     print "MENU: <@arguments>\n";
     $lobj->add(@arguments)
    } else {
     $is_toplevel || $crail->{pack}[0] && packpush($_, $pack, \@arguments);
    }

   }
 }

  $is_toplevel || $is_menu || $crail->{pack}[0] && packnow($crail, $leveltype, $lvname, $lobj, $pack);
 
 return {name=>$lvname, obj=>$lobj}
}


sub tixgeneric {
my ($crail, $widget_args, $leveltype, $lvname, $lobj, $parent) = @_;

# print "tixgeneric: leveltype<$leveltype> lvname<$lvname> lobj<$lobj>\n";

 my $is_tixbuttonbox_tixselect                     = $leveltype =~ /tixbuttonbox|tixselect/io;
 my $is_tixpanedwindow_tixnotebook_tixlistnotebook = $leveltype =~ /tixpanedwindow|tixnotebook|tixlistnotebook/io;
 my $is_tixpanedwindow                             = $leveltype =~ /tixpanedwindow/io;
 my $is_tixlistnotebook                            = $leveltype =~ /tixlistnotebook/io;
 my $is_tixlabelframe_tixscrolledwindow            = $leveltype =~ /tixlabelframe|tixscrolledwindow/io;
 my $is_tixballoon                                 = $leveltype =~ /tixballoon/io;

 my %page_or_pane;
 my $pack = [];
 foreach (@$widget_args) {
  next if ref($_) && $$_[0] =~  /$$crail{config}{preprocess}{special}/o;

  if (exists $$crail{config}{handler}{$$_[0]}) {
   if($$crail{config}{skip}{$$_[0]}) {
    unshift @{$crail->{pack}}, 1;
    tkwalk($crail, $_, $lobj);
    shift @{$crail->{pack}};
    next
   }

   #
   # Only when the current widget does not have all of its
   # sub-widgets built-in. In that case we need to define them
   # first, before use
   #
   if ($is_tixbuttonbox_tixselect) {
    # tixbuttonbox | tixselect
    my $slvinfo = get_slv_attributes($_);

    $lobj->add($$slvinfo{slvname});
    $lobj->subwidget($$slvinfo{slvname}, 'configure', @{$$slvinfo{options}});

   } elsif ($is_tixpanedwindow_tixnotebook_tixlistnotebook) {
    # tixpanedwindow | tixnotebook | tixlistnotebook
    my $subwidget_name = $$_[1][0];
    if ($is_tixlistnotebook) {
	    #my $hlist = $lobj->subwidget('hlist')->add($subwidget_name, -text=>$subwidget_name);
	    my $hlist = $lobj->subwidget('hlist');
	    Tkx::eval("$hlist add $subwidget_name -text $subwidget_name");
    } 

    #$$crail{tcl}->Eval("$lobj add $subwidget_name");
    $lobj->add($subwidget_name);

    #my $subwidget_path = $$crail{tcl}->Eval("$lobj subwidget $subwidget_name");
    my $subwidget_path = $lobj->subwidget($subwidget_name);
    $page_or_pane{$subwidget_path}  = 1;

    push @{$$crail{namelifo}}, $subwidget_name;

#    print q($$crail{winfo}{').join('.',  @{$$crail{namelifo}}).q('})."$$crail{tcl}->widget($subwidget_path)\n";
    ${eval q(\$$crail{winfo}{').join('.',  @{$$crail{namelifo}}).q('})} = Tkx::widget->new($subwidget_path);

    # The current Pane or Page is now seen as a parent. tkwalk will now build and link/attach a full widget Tree to it.
    unshift @{$crail->{pack}}, 1;
    tkwalk($crail, $_, Tkx::widget->new($subwidget_path));
    shift @{$crail->{pack}};

    pop @{$$crail{namelifo}};
   } elsif ($is_tixlabelframe_tixscrolledwindow) {
    my $subframe_name = $leveltype =~ /tixlabelframe/io ? 'frame' : 'window';
    #my $subframe_path = $$crail{tcl}->Eval("$lobj subwidget $subframe_name");
    my $subframe_path = $lobj->subwidget($subframe_name);

    # The current sub-frame is now to be considered as a parent. tkwalk will now build and link/attach a full widget Tree to it.
    unshift @{$crail->{pack}}, 1;
    tkwalk($crail, $_, Tkx::widget->new($subframe_path));
    shift @{$crail->{pack}};
   }

  } else {
   my $is_tixcombobox = $$_[0] =~ /history/io;
   # Specific to *tixcombobox*
   $is_tixcombobox && do {$lobj->appendhistory($_) foreach (@{$$_[1]})};
   
   # This part is not useful for 'tixcombobox' widgets
   my %options= get_opt();
   my @options;
   do {@options = map {-$_ => (/$$crail{config}{cmd_and_var_re}/o ? eval do {
				                                      $options{$_} =~ s/([\$\&])(?=[a-zA-Z])/$1main::/g; 
								      "$options{$_}"
			                                              } : $options{$_})} keys %options} unless $is_tixcombobox;
   
					      #$$crail{tcl}->Eval("$lobj bind $parent". join('', map {" -$_ {$options{$_}}"} keys %options)) if $is_tixballoon && $$_[0] =~ /bind/io;
   $lobj->bind($parent, map {'-'.$_=>$options{$_}} keys %options) if $is_tixballoon && $$_[0] =~ /bind/io;

   # Part for handling built-in sub-widget(s)
   if (my ($subw) = $$_[0] =~ /\.(\S+)/o) {
     my $last_path = $lobj;
     foreach (split(/\./, $subw)) {$last_path = $last_path->subwidget($_)}


     unless (exists $page_or_pane{$last_path}) {
	     Tkx::widget->new($last_path)->configure(@options)
     } else {
      my $configure = ($is_tixpanedwindow ? 'pane' : 'page') . 'configure';

      $lobj->$configure($subw, @options)
     }
   }

   $is_tixballoon || $crail->{pack}[0] && packpush($_, $pack, \@options);
  }
 }

 $is_tixballoon || $crail->{pack}[0] && packnow($crail, $leveltype, $lvname, $lobj, $pack);

 return {name=>$lvname, obj=>$lobj}
}


sub tkpopupmenu {
my ($crail, $args, $leveltype, undef, undef, $parent) = @_;

# print "tkpopupmenu <$leveltype> <$$args[0]>\n";
 push @{$$crail{popup}}, {widget=> join('.',  @{$$crail{namelifo}}), wobj=>$parent, menu=> $$crail{gui}[0]{data}{$$args[0]}}
}

sub grid {
my ($crail, $args, $leveltype, undef, undef, $parent) = @_;

# print "grid: leveltype<$leveltype> lvname<$$args[0]>\n";

 my $ccol      = -1;
 my $crow      = -1;
 my $row_seen  = 0; 
 foreach (@$args) {
  unless (ref) {
   if ($_ eq 'next-row') {
    next if $crow < 0;

    ++$crow;
    $ccol     = -1;
   } elsif ($_ eq '-') {
    ++$ccol;
   }

   next
  }

  ++$ccol;
  ++$crow if $crow < 0;
  unshift @{$crail->{pack}}, 0;

  unshift @{$crail->{grid}}, {row=>$crow, col=>$ccol};
  my $cw = tkwalk($crail, $_, $parent)->{obj};
  shift @{$crail->{grid}};

  shift @{$crail->{pack}};

#  print "(".$crail->{tcl}->Eval("winfo class $cw").") grid -column $ccol -row $crow $cw\n";
  Tkx::grid($cw, -column=>$ccol, -row=> $crow) unless $$_[0] eq 'sticky';

 }

# print "grid: Leaving\n";
 return undef
}

sub sticky {
my ($crail, $args, $leveltype, undef, undef, $parent) = @_;

# print "sticky: leveltype<$leveltype> style<$$args[0]>\n";
 my $cw = tkwalk($crail, $args->[1], $parent)->{obj};

 my $ccol = $crail->{grid}[0]{col};
 my $crow = $crail->{grid}[0]{row};

 Tkx::grid($cw, -sticky=>$$args[0], -column=>$ccol,  -row=>$crow);
}


sub maxsize {
my ($crail, $args, $leveltype, undef, undef, $parent) = @_;

# print "maxsize: leveltype<$leveltype> style<$$args[0]>\n";
 my $cw = tkwalk($crail, $args->[2], $parent)->{obj};


 Tkx::wm('maxsize', $cw,  @$args[0 .. 1]);
}

sub minsize {
my ($crail, $args, $leveltype, undef, undef, $parent) = @_;

# print "minsize: leveltype<$leveltype> style<$$args[0]>\n";
 my $cw = tkwalk($crail, $args->[2], $parent)->{obj};


 Tkx::wm('minsize', $cw,  @$args[0 .. 1]);
}

sub resizable {
my ($crail, $args, $leveltype, undef, undef, $parent) = @_;

# print "resizable: leveltype<$leveltype> style<$$args[0]>\n";
 my $cw = tkwalk($crail, $args->[2], $parent)->{obj};


 Tkx::wm('resizable', $cw,  @$args[0 .. 1]);
}

sub size {
my ($crail, $args, $leveltype, undef, undef, $parent) = @_;

# print "size: leveltype<$leveltype> style<$$args[0]>\n";
 my $cw = tkwalk($crail, $args->[2], $parent)->{obj};


 Tkx::wm('maxsize',   $cw,  @$args[0 .. 1]);
 Tkx::wm('minsize',   $cw,  @$args[0 .. 1]);
 Tkx::wm('resizable', $cw,  0,  0);
}

sub xform_grid {
my ($crail, $args, $leveltype, undef, undef, $parent) = @_;

 Tkx::wm('maxsize',  $parent, @$args[0 .. 1]);
 kx::wm('minsize',   $parent, @$args[0 .. 1]);
 kx::wm('resizable', $parent, 0,  0);
 kx::tixForm('grid', $parent, @$args[0 .. 1]);
}

sub xform_attach {
my ($crail, $args, $leveltype, undef, undef, $parent) = @_;

 my %options = map {split /:/} grep {! ref} @$args;
 my $options = join(" ", map {"-$_ $options{$_}"} keys %options);

 # Disable the pack geometry manager for childrens
 unshift @{$crail->{pack}}, 0;
 Tkx::tixForm('configure', $_, map {"-".$_, $options{$_}} keys %options) foreach (map {tkwalk($crail, $_, $parent)->{obj}} grep {ref} @$args);
 shift @{$crail->{pack}};
}

sub event {
my ($crail, $args, $leveltype, undef, undef, $parent) = @_;

# [0] == TYPE,  Enter/Leave/FocusIn/FocusOut/...
# [1] == CALLBACK name
# [2] == FILTER   (optional)
my ($type, $callback, $filter) = @$args;

#print "EVENT HANDLER <$type> <$callback> <$filter> parent ($parent)\n";
 push @{$$crail{event}}, {widget=> join('.',  @{$$crail{namelifo}}), wobj=>$parent, type=>$type,  callback=> "main::".$callback, filter=> ($filter || ".*")}
}

sub EventAttach {event_attach(@_)}

sub event_attach {
my ($crail) = @_;

# print "popup_recurse: Attaching popups..\n";
 foreach my $cpopup (@{$$crail{popup}}) {
  popup_recurse($crail, $cpopup->{wobj}, tkwalk($crail, $cpopup->{menu}, $cpopup->{wobj}))
 }
# print "popup_recurse: Done.\n";

# print "event_recurse: Attaching EVENTs (aka Focus in) events..\n";
 foreach my $cevent (@{$$crail{event}}) {
  event_recurse($crail, $cevent->{wobj}, $cevent->{type}, $cevent->{callback}, qr{$cevent->{filter}})
 }
 
 @{$$crail{event}} = ();
 @{$$crail{popup}} = ();
# print "event_recurse: Done.\n";
}

sub popup_recurse {
my ($crail, $topwidget, $menu) = @_;

# my $popup = $menu; 

# print "popup_recurse: <$topwidget>\n";
 my @childrens = map {Tkx::widget->new($_)} split /\s+/, Tkx::winfo('children', $topwidget);
 unless (@childrens) {
  # Leaf widget reached
  $topwidget->g_bind('<Button-3>', [sub {$$menu{obj}->g_tk___popup($_[0], $_[1]); Tkx::set("crail(widget)", $_[2])}, Tkx::Ev("%X", "%Y", "%W")]);
  return
 }
 
 # Try to attach to childrens first, if any
 popup_recurse($crail, $_, $menu) foreach (@childrens);
 
 # Then to the parent
 $topwidget->g_bind('<Button-3>', [sub {$$menu{obj}->g_tk___popup($_[0], $_[1]); Tkx::set("crail(widget)", $_[2])}, Tkx::Ev("%X", "%Y", "%W")]);
}

sub event_recurse {
my ($crail, $topwidget, $type, $callback, $filter) = @_;
#print "event_recurse0: topwidget<$topwidget> filter<$filter>\n";
 
# print "event_recurse1: topwidget<$topwidget>\n";
 my @childrens = map {Tkx::widget->new($_)} split /\s+/, Tkx::winfo('children', $topwidget);
 unless (@childrens) {
  # Leaf widget reached
  return unless "$topwidget" =~ /$filter/;
  $topwidget->g_bind("<$type>", [sub {&$callback(@_)}, Tkx::Ev("%W")]);
  return
 }
 
 # Try to attach to childrens first, if any
 event_recurse($crail, $_, $type, $callback, $filter) foreach (@childrens);
 
 # Then to the parent
 return unless "$topwidget" =~ /$filter/;
 $topwidget->g_bind("<$type>", [sub {&$callback(@_)}, Tkx::Ev("%W")]);
}

# This function HAS NOT BEEN PORTED to LISPISH yet, 
#
# Now options/pack/.\w+ generate ARRAY REFs, so are not to be considered as simple scalar string anymore !!!
sub optionmenu {
my ($crail, $widget_args, $leveltype, $lvname, $lobj, $parent) = @_;

 my $pack=[];
 my %omenu_hash;
 my $varname= $lvname;
 my $menuname= $lvname."_m";
 my @moptions;
 my @options;
 
 foreach (@$widget_args) {
  if(/^\s*valist\b/) {
   my @wlist= /(?:".+?(?<!\\)"|\S+)/go;
   my $type = shift @wlist;
   
   push @{$omenu_hash{$type}}, @wlist;
  } else {
   my %options= get_opt();
   my $options= join("", map {" -$_ $options{$_}"} keys %options);
   my @arguments=map {-$_ => $options{$_}} keys %options; 

   /^\s*moptions\b/o && push @moptions, "\${$menuname}\tconfigure  $options\n";
   /^\s*options\b/o  && push @options,  "$parent.$lvname\tconfigure $options\n";
   /^\s*pack\b/o     && push @$pack, @arguments
  }
 }

 
 my $tk_optionmenu = "set $menuname [tk_optionMenu $parent.$lvname $varname @{$omenu_hash{valist}}]";
 Tkx::eval($tk_optionmenu);

 @moptions && Tkx::eval("@moptions");
 @options  && Tkx::eval("@options");

 my $packing="pack\t$parent.$lvname ". join(' ', @$pack);
 Tkx::eval($packing)
}


# This function HAS NOT BEEN PORTED to LISPISH yet, 
#
# Now options/pack/.\w+ generate ARRAY REFs, so are not to be considered as simple scalar string anymore !!!
sub tixoptionmenu {
my ($crail, $widget_args, $leveltype, $lvname, $lobj) = @_;

 my $pack = [];
 my $num=0;

 foreach (@$widget_args) {
  if (ref) {
   my $slvinfo = get_slv_attributes($_);

   my $configure_subwidget = $lobj->path."\tsubwidget\t$$slvinfo{slvname} configure ". join(" ", @{$$slvinfo{options}});

   Tkx::eval($configure_subwidget);
  } else {
    my ($type)      = /(\S+)/o;
    next if $type   =~ /options/io;
    
    my %options     = get_opt();
    my @options_lst = map {-$_ => $options{$_}} grep {!/label/io} keys %options;
    my ($label)     = map {$options{$_}} grep { /label/io } keys %options;

    unless (/^\s*\.(\S+)/o) {
     $label = $label || "s$num";

     $label && unshift @options_lst, $label;
     unshift @options_lst, $type;

     $lobj->add(@options_lst);
    }
    
   /^\s*\.(\S+)/o &&
   Tkx::eval(join(' subwidget ', $lobj, map {split /\./} /^\s*\.(\S+)/o). " configure ". join(" ", @options));

    packpush($_, $pack, \@options);
    ++$num
  }
 }

 packnow($crail,$leveltype, $lvname, $lobj, $pack);
 
 return {name=>$lvname, obj=>$lobj} 
}


sub tixpopupmenu {
my ($crail, $widget_args, $leveltype, $lvname, $lobj, $parent) = @_;

 my $pack = [];
 foreach (@$widget_args) {
  if (ref) {
   my $slvinfo = get_slv_attributes($_);
   my $name    = $$slvinfo{slvname};
   my $path    = $lobj->subwidget($name);
   my $class   = lcfirst Tkx::winfo('class', $path);

   #print "tixpopupmenu - name=$name path=$path class=$class\n";

   $lobj->subwidget($name, 'configure', @{$$slvinfo{options}});

   my $handler = $$crail{config}{handler}{$class} || $class;
   &$handler($crail, $_, undef, Tkx::widget->new($path), $class);
  } else {
   my %options= get_opt();
   my @options= map {-$_ => $options{$_}} keys %options;
   
   packpush($_, $pack, \@options);
  }
 }


 $lobj->bind($parent);
 #$$crail{tcl}->Eval("$lobj post $parent 10 10");

 return {name=>$lvname, obj=>$lobj}
}


sub get_slv_attributes {
my ($gsa) = @_;

 my %goptions;
 foreach (@{$$gsa[1]}) {
  unless(ref $_) {
   $goptions{slvname} = $_;
   next
  }

  my $type = $$_[0];

  my %options= get_opt();
  my @options= map {-$_ => (/$$crail{config}{cmd_and_var_re}/o ? eval do {
								  $options{$_} =~ s/([\$\&])(?=[a-zA-Z])/$1main::/g;
								  "$options{$_}"
							         }: $options{$_} )} keys %options;

  $type =~ /options/io && push @{$goptions{$type}}, @options 
 }

 return {%goptions}
}

sub packpush {
my ($itemtype, $pack, $options) = @_;

  push @$pack, @$options if $$itemtype[0] =~ /pack/io;
}

sub packnow {
my ($crail, $leveltype, $lvname, $lobj, $pack) = @_;

 # print "PACK $leveltype $lvname<@$pack>$lobj\n";
 $lobj->g_pack(@$pack)
}

sub widget_call {
my ($crail, $args, $leveltype, undef, undef, $parent) = @_;

 my $call_arg = $$args[0];
 
 my @splitarg = split /\//, $call_arg;

 if ($call_arg =~ /\//) {
  if ($call_arg =~ /^\//) {
   shift @splitarg;
   if (@splitarg >= 3) {
    my $widget_name = pop @splitarg;
    my $tkfile = '/'.join("/", @splitarg).".tk";

    my $callgui = TkGui::Get($tkfile);
    if (exists $callgui->{$widget_name}) {
     unshift @{$crail->{gui}}, {data=>$callgui, file=>$tkfile};
    
     my $retv = tkwalk($crail, $callgui->{$widget_name}, $parent);
    
     shift @{$crail->{gui}};

     return $retv;

    } else {
     die "(tkinit)(widget_call) -E- Can't find Widget '$widget_name' in '$tkfile',"
    }

   } else {
    die "(tkinit)(widget_call) -E- Full Widget path specification should be of the form '/PATH/FILE_BASENAME/WIDGET_NAME',"
   }
  } elsif (@splitarg == 2) {
   my ($basename, $widget_name) = @splitarg;
   my $callgui                  = TkGui::Get(my $tkfile = PathSearch->go($basename, 'tk'));
   
   if (exists $callgui->{$widget_name}) {
    unshift @{$crail->{gui}}, {data=>$callgui, file=>$tkfile};
    
    my $retv = tkwalk($crail, $callgui->{$widget_name}, $parent);
    
    shift @{$crail->{gui}};

    return $retv

   } else {
    die "(tkinit)(widget_call) -E- Can't find Widget '$widget_name' in '$tkfile',"
   }

  } else {
   die "(tkinit)(widget_call) -E- Partial Widget path specification should be of the form 'FILE_BASENAME/WIDGET_NAME',"
  }
 } else {
  if (exists $crail->{gui}[0]{data}{$call_arg}) {
   tkwalk($crail, $crail->{gui}[0]{data}{$call_arg}, $parent);
  } else {
   die "(tkinit)(widget_call) -E- Can't find Widget '$call_arg' in '$crail->{gui}[0]{file}',"
  }
 }

}



1;
