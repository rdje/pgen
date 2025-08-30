#! env groovy

import java.util.regex.*

class LangSpec {
 def cbrace_index = 7
 def spec_descr   = [
  [ handler: {descr, string, gdata ->
   
    def specentry = []
    def specs     = []

    while (1) {
     def minfo = Dispatch(string.m, gdata.startREs)

     if (minfo == null) {
       if (specentry.size()) specs << specentry 
       return specs
     }

println "Found:======> <${minfo.pos + 1}> <$minfo.group>"
     def retv = descr[minfo.pos + 1].handler.call(minfo, descr, string, gdata)
     if (retv == null) return null

     if (retv[0] != 'COMMENT') {
      if (retv[0] =~ /ELABEL/) {
       if (specentry.size()) {
	specs << specentry
        specentry = [retv]
       } else {
        specentry << retv
       }
      } else {
        specentry << retv
      }
     }
    }
  }],

  [// Entry Label
    re      : [~/\w+::?/],
    handler : {info, descr, string, gdata -> 
println "Entry Label: <$info.group>"
                info.group.replaceAll(':', '')
                def target =  info.group =~ /:/ ? '_INITIAL' : ''
		info.group.replaceAll(':', '')
		return ["ELABEL$target", info.group]
    }
  ],

  [// RE pattern
     re      : [~/(?<!\\)\/.+?(?<!\\)\//],
     handler : {info, descr, string, gdata -> info.group.replaceAll('^/|/$', ''); println "RE pattern: <$info.group>"; ["RE", info.group]}
  ],

  [// Action code block
     re      : [~/->\s*\w+(?:\[\d+\])?\s*\{/, ~/\}/],
     handler : {info, descr, string, gdata ->
               def ipos = string.m.end()

println "Action code block: <$info.group>"
               def m1          = info.group =~ /(\w+)(?:\[(\d+)\])?/
               def entry_label = m1[0][1]
               def reidx       = m1[0][2] != null ? m1[0][2] : 0

               while (1) {
                def minfo = Dispatch(string.m, gdata.cbrace)
                if (minfo == null) return null

                switch (minfo.pos) {
                 case 1 : return ['ACODE', [relabel:entry_label, reidx:reidx, code:string.v.substring(ipos, minfo.end-1)]]
         	 case 0 : descr[cbrace_index].handler.call(minfo, descr, string, gdata)
                }
               }
     }
  ],

  [// Empty Action code block
     re      : [~/->\s*\w+(?:\[0\])?/],
     handler : {info, descr, string, gdata -> 
     
println "Empty Action code block: <$info.group>"
               def m1          = info.group =~ /(\w+)/
               def entry_label = m1[0][1]
               ['ACODE', [relabel:entry_label, reidx:0, code:"call($entry_label)"]]
     }
  ],


  [// Non-Action code block
     re      : [~/\w+\s*\{/, ~/\}/],
     handler : {info, descr, string, gdata ->
               def ipos = string.m.end()
println "Non-Action code block: <$info.group>"
               def m1    = info.group =~ /(\w+)/
               def type  = m1[0][1]
               def reidx = m1[0][2] != null ? m1[0][2] : 0

               while (1) {
                def minfo = Dispatch(string.m, gdata.cbrace)
                if (minfo == null) return null

                switch (minfo.pos) {
                  case 1 : return ["${type}CODE", string.v.substring(ipos, minfo.end-1)]
         	  case 0 : descr[cbrace_index].handler.call(minfo, descr, string, gdata)
                }
               }
     }
  ],

  [// Comment
     re      : [~/(?:\r\n?)?[ \t]*#.*/],
     handler :  {info, descr, string, gdata -> println "Comment: <$info.group>"; ["COMMENT"]}
  ],

  [// Curly Brace
     re      : [~/(?<!\\)\{/, ~/(?<!\\)\}/, ~/(?<!\\)".*?(?<!\\)"/, ~/(?<!\\)'.*?(?<!\\)'/],
     handler : {info, descr, string, gdata ->

println "Curly Brace: <$info.group>"
               while (1) {
                def minfo = Dispatch(string.m, gdata.cbrace)
                if (minfo == null) return null

                switch (minfo.pos) {
                  case 1 : return 1
         	  case 0 : descr[cbrace_index].handler.call(minfo, descr, string, gdata)
                }
               }
     }
  ]
 ]

 def gdata = [
     startREs : spec_descr.findAll {it?.re != null}.collect {it.re[0]},
     cbrace   : spec_descr[cbrace_index].re
 ]

 def LangSpec (specfile) {
  def stringv = (new File(specfile)).getText()
  def string  = [v:stringv, m:stringv =~ /./]

  def langobj = spec_descr[0].handler.call(spec_descr, string, gdata)
 }

 def initial_target = '_main_'
 def info_main      = [:]
 def live_spec_descr (langobj) {
  langobj.collect {}.
 }

 def last_dispatch_pos = 0
 def Dispatch (m, plist) {
  def matchlist = [];
  plist.eachWithIndex {p, idx -> 
    if(m.usePattern(p).find(last_dispatch_pos)) {
      matchlist << [pos:idx, group:m.group(), start:m.start(), end:m.end()]
     }
  }

  if (matchlist.size() == 0) return null

  def startlist  = []
  def min_start = null
  matchlist.each {
   min_start = min_start.equals(null) || min_start > it.start ? it.start : min_start
   if(startlist[it.start] == null) {
    startlist[it.start] = [it]
   } else {
    startlist[it.start] << it
   }
  }
  
  def max_end = 0 
  def endlist   = []
  startlist[min_start].each {
   if (it.end > max_end)  max_end = it.end
   if(endlist[it.end] == null) {
    endlist[it.end] = [it]
   } else {
    endlist[it.end] << it
   }
  }

  last_dispatch_pos = endlist[max_end][0].end
  return endlist[max_end][0]
 }
}


myrun = new LangSpec("specs/lib_reader.spec")
