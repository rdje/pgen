require 'java'
require 'Hash'

include_class  'java.util.regex.Pattern'

class REDispatch 
 def REDispatch.fire(m, plist)
	 puts "REDispatch: ================ Entering ==============="
   matchlist = []
   regionst  = m.regionStart
   plist.each_with_index {|p, idx| 
     puts "pattern<#{p}>"
     matchlist << {'pos' => idx, 'group' => m.group(), 'start' => m.start(), 'end' => m.end()} if m.usePattern(p).find(regionst)
   }

   return nil if matchlist.empty?
   
   startmap = {}
   matchlist.each {|it| (startmap[it['start']] ||= [])  << it}

   endmap  = {}
   startmap[startmap.keys.sort.first].each {|it| (endmap[it['end']] ||= []) << it}

   lastkey = endmap.keys.sort.last
   m.region(endmap[lastkey].first['end'], m.regionEnd)
   return endmap[lastkey].first
 end
end


class LinkedSpec
 @@cbrace_index = 9
 @@node_type     = {
	'&'       => 'AND',
	'|'       => 'OR',
	'+'       => 'REP_PLUS',
	'*'       => 'REP_STAR',
	'?'       => 'REP_OPT'
 }

 @@rep_nodes_minmax = {
 	'REP_PLUS'=> [1, 10**9],
 	'REP_STAR'=> [0, 10**9],
 	'REP_OPT' => [0, 1]
 }

 @@spec_descr = [
  {# Spec			-0-
   'handler' => lambda {|descr, string, gdata|
     specentry = []
     specs     = []

     loop do
      minfo = REDispatch.fire(string['m'], gdata['startres'])
      unless minfo
	puts "(Spec) Closing specentry DUE TO EOF\n" unless specentry.empty?
        specs << specentry unless specentry.empty?
	return specs
      end

      puts "################## FOUND[#{minfo['pos'] + 1}] #########################";
      retv = descr[minfo['pos'] + 1]['handler'].call(minfo, descr, string, gdata)
      return nil unless retv

      unless retv.first == "COMMENT"
       if retv.first =~ /ELABEL/o
        puts "ELABEL<"+retv[0]+">"
	unless specentry.empty?
         specs << specentry
	 specentry = [retv]
	else
         specentry << retv
	end
       else
	puts "(Spec) Pushing in specentry (#{retv * ' '})\n"
        specentry << retv
       end
      end
     end  
   }
  },

  {# Entry label		-1-
   're'=> [/\w+::?(?:&|\||\+|\*|\?)?/o].map {|it| Pattern.compile(it.source)},
   'handler' => lambda {|info, descr, string, gdata|
     puts  "Entry Label: <"+info['group']+">"

     info['group'].sub!(/:/o, "")
     target =  info['group'].match(/:/o) ? "_INITIAL" : ""
     info['group'].sub!(/:/o, "")
     info['group'].sub!(/(\W)/o, "")
     puts "Entry Label -BIS: <"+info['group']+">"
     gdata["_current_entry"] = info['group']
     ["ELABEL"+target, info['group'], $1 ? @@node_type[$1] : 'default']
   },
  },

  {# RE pattern                 -2-
   're'=> [/(?<!\\)\/.+?(?<!\\)\//o].map {|it| Pattern.compile(it.source)},
   'handler' => lambda {|info, descr, string, gdata|
     info['group'].gsub!(/^\/|\/$/o, "")
     puts "RE pattern: <"+info['group']+">"
     ["RE", info['group']]
   }
  },

  {# Action code block          -3-
   're'=> [/->\s*\w+(?:\[\d+\])?\s*\{/o, /\}/o].map {|it| Pattern.compile(it.source)},
   'handler'=> lambda {|info, descr, string, gdata|
      ipos = info['end']
 
      puts "Action code block: <"+info['group']+">"
      entry_label, reidx = info['group'].match(/(\w+)(?:\[(\d+)\])?/o).captures
      #entry_label = getMatch(1)
      #reidx       = getMatch(2) != null ? getMatch(2) : 0
      reidx ||= 0
 
      loop do
       minfo = REDispatch.fire(string['m'], gdata['cbrace'])
       return nil unless minfo
 
       case minfo['pos'] 
        when 0 : descr[@@cbrace_index]['handler'].call(minfo, descr, string, gdata)
        when 1 : return ["ACODE", {"relabel" => entry_label, "reidx" => reidx.to_i, "code"=>string['v'][ipos .. minfo['end']-2]}]
	end
      end
   }
  },

  {# Empty Action code block    -4-
   're'=> [/->\s*\w+(?:\[0\])?/o].map {|it| Pattern.compile(it.source)},
   'handler'=> lambda {|info, descr, string, gdata|
         print("Empty Action code block: <"+info['group']+">")
         entry_label = info['group'].match(/(\w+)/o).captures.first
         ["ACODE", {"relabel" => entry_label, "reidx" => 0, "code" => "specall("+entry_label+")"}]
   }
  },

  {# Non-Action code block      -5-
   're'=> [/\w+\s*\{/o, /\}/o].map {|it| Pattern.compile(it.source)},
   'handler'=> lambda {|info, descr, string, gdata|
         puts "Non-Action code block: <"+info['group']+">"
	 ipos = info['end']
 
         type = info['group'].match(/(\w+)/o).captures.first
 
	 puts "TYPE<#{type}>"
         loop do
          minfo = REDispatch.fire(string['m'], gdata['cbrace'])
	  return nil unless minfo
 
          case minfo['pos']
	   when 0 : descr[@@cbrace_index]['handler'].call(minfo, descr, string, gdata)
	   when 1 : return [type+"CODE", string['v'][ipos .. minfo['end']-2]]
	  end
	 end
   }
  },

  {# Comment                    -6-
   're'=> [/(?:\r\n?)?[ \t]*#.*/o].map {|it| Pattern.compile(it.source)},
   'handler'=> lambda {|info, descr, string, gdata| puts "Comment: <"+info['group']+">"; ["COMMENT"]}
  },

  {# Blind call code block      -7-
   're'=> [/=>\s*\w+\s*\{/o, /\}/o].map {|it| Pattern.compile(it.source)},
   'handler'=> lambda {|info, descr, string, gdata|
      ipos    = info['end']
      specall = info['group'].match(/(\w+)/o).captures.first

      puts "(Blind call code block)(#{info['group']})(#{specall}, #{ipos})"
      loop do
       minfo = REDispatch.fire(string['m'], gdata['cbrace'])
       return nil unless minfo

       case minfo['pos']
	when 1
          # Closing brace, recursion stops here
          puts "(Action code block) (#{string['v'][ipos .. minfo['end'] - 2]}) Closing"
	  return ['BCODE', {'call'=>specall, 'code'=>"#{gdata["_current_entry"]} = specall(#{specall})\n" + string['v'][ipos .. minfo['end'] - 2]}]
	when 0
          # Opening brace found, triggering recursion
          puts "(Curly BRACE) Recursion"
          descr[@@cbrace_index]['handler'].call(minfo, descr, string, gdata)
          puts "(Curly BRACE) Back From Recursion"
        else puts "QUOTES <#{info['group']}>"
       end
      end
   }
  },

  {# Empty Blind code block      -8-
   're'=> [/=>\s*\w+/o].map {|it| Pattern.compile(it.source)},
   'handler'=> lambda {|info, descr, string, gdata|
         specall = info['group'].match(/(\w+)/o).captures.first 
         puts "(Empty Action code block) (#{entry_label}) (#{specall})"
         return ['BCODE', {'call'=> specall, 'code'=>"#{gdata["_current_entry"]} = specall(#{specall})"}]
   }
  },

  {# Curly Brace      -9-  + dquotes + squotes
   're'=> [/(?<!\\)\{/o, /(?<!\\)\}/o, /(?<!\\)".*?(?<!\\)"/o, /(?<!\\)'.*?(?<!\\)'/o].map {|it| Pattern.compile(it.source)},
   'handler'=> lambda {|info, descr, string, gdata|
         puts "Curly Brace: <"+info['group']+">"
         loop do
          minfo = REDispatch.fire(string['m'], gdata['cbrace'])
	  return nil unless minfo
 
          case minfo['pos'] 
	   when 0 :  descr[@@cbrace_index]['handler'].call(minfo, descr, string, gdata)
	   when 1 :  return 1
           else   puts "QUOTES <#{info['group']}>"
	  end
	 end
   }
  }
 ]

 @@gdata = {
   'startres' => @@spec_descr.select {|it| it.has_key?('re')}.map {|it| it['re'].first}, 
   'cbrace'   => @@spec_descr[@@cbrace_index]['re']
 }


 def get (*args) 
   retv = @@spec_descr[0]['handler'].call(@@spec_descr, {'v'=>args[0], 'm'=>Pattern.compile('.').matcher(args[0])}, @@gdata)
   
   option = Hash[*args[1 .. args.length-1]]
   @drive = option['drive']

   puts "<#{option.to_a.flatten * "\n"} [#{@drive}]>"
   # _main_ is the default target 
   auto_descr_spec  = spec_descr(retv);
   # $$auto_descr_spec{_main_} = spec_main_handler($info_main);
   # my $final_descr           = {spec=>$auto_descr_spec, gdata=>spec_gdata($auto_descr_spec)};

   # puts "\n\nsub Get {&{\$descr->{spec}{$initial_target}}(\$descr, \$_[0])}" if $pm_drive;

   # return sub {&{$final_descr->{spec}{$initial_target}{handler}}($final_descr, $_[0])}
 end 


 def spec_descr (specretv, initial_target="_main_", info_main={}) 
 
  puts 'my $descr = {
  spec => {' if @drive;
 
  specinfo = specretv.map {|it| spec_entry(it, initial_target, info_main)}
 
  #print "},\n";
 
  Hash[*specinfo]
 end

 def spec_entry (einfo, initial_target, info_main)
    info      = {}
    label     = ""
    node_type = ""
    res       = []
    icode     = ""
    ecode     = ""
    excode    = ""
    itcode    = ""
    lxcode    = ""
    lscode    = ""
    lecode    = ""
    aCODEs    = []
    bCODEs    = {}
    bCALLs    = []
    gDATA     = []
    ab_count  = {}
    handlers  = {}

    einfo.each {|it|
     case it[0]
      when "ACODE"
            aCODEs << call_spec_handler_subst(it[1]['code'])
            gDATA  << {'label' => it[1]['relabel'], 'idx' => it[1]['reidx']}
	    ac_count['ACODE'] = ac_count['ACODE'] ? (ac_count['ACODE']+1) : 1

      when "BCODE"
            bCALLs << it[1]['call']
            bCODEs[it[1]['call']] = call_spec_handler_subst(it[1]['code'])
	    ac_count['BCODE']     = ac_count['BCODE'] ? (ac_count['BCODE']+1) : 1

      when "ICODE"  : icode  = it[1]
      when "ECODE"  : ecode  = it[1]
      when "EXCODE" : excode = it[1]
      when "ITCODE" : itcode = it[1]
      when "LXCODE" : lxcode = it[1]
      when "LSCODE" : lscode = it[1]
      when "LECODE" : lecode = it[1]
      when "RE"     : res << Pattern.compile(it[1])
      when /ELABEL/o 
	  label, node_type = it[1 .. 2]
	  if it[0] =~ /ELABEL_INITIAL/o
            puts "(LinkedSpec) -E- Only one initial target may be specified, 1st='#{initial_target}' 2nd='#{it[1]}'"
	    exit
	  end
     end
    }

   if !ab_count['ACODE'].empty? && !ab_count['BCODE'].empty?
     puts "\n-E- Entry '#{label}' should not have both ACTION (->) and BLIND CALL (=>) codes\n";
     exit
   end


   # Initial value of the handler code
   actual_icode  = icode  && "#{icode};" || ""
   actual_ecode  = ecode  && "#{ecode}"  || ""
   actual_excode = excode && "#{excode}" || ""
   actual_itcode = itcode && "#{itcode}" || ""

   handler = "|descr, sTRING, info|

   imatch = info['group']
   iindex = info['pos']
   iend   = info['end']

   #{actual_icode}"

   notvalid_lcodes = %r/^\s*$/o
   acodes = ""
   bcodes = ""
   if !aCODEs.empty? || !bCODEs.empty? || lxcode !~ notvalid_lcodes || lscode !~ notvalid_lcodes || lecode !~ notvalid_lcodes
     acodes += aCODEs.empty? ? "" : "\ncase minfo['pos']\n  #{wl = []; aCODEs.each_with_index {|v, i| wl << %Q{when #{i} \n  #{v}\n  }}; wl.join} end\n"
     bcodes += bCALLs.empty? ? "" : "\ncase call\n  #{wl = []; bCALLs.each {|v| wl << %Q{when #{v} \n  #{bCODEs[v]}\n  }}; wl.join} end\n"
   end
 end
end
