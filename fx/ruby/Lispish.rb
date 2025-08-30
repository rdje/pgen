class Lispish
  attr_writer :debug

  @@debug = false

  @@comment = /;.*?\n/
  @@lispish = /\(|\)|#@@comment/
  @@dquote  = /".*?"/
  @@curlyb  = /\{|\}/
  @@space   = /\s+/
  @@others  = /[^\s"\{\}\(\);]+/
  @@group   = /\(|#@@dquote|\{|#@@space|#@@others|#@@comment|\)/

 def initialize (string) 
  @ms = Mstring.new(string)
 end

 def next
  while true
   return nil unless md   = @ms.find(@@lispish)
   return nil unless retv = case md[0]
                               when "(" then group
                               when ")"
                                puts "(Lispish::next) -E- Syntax error" if @@debug
                                exit
                               # Comments
                               else comment
                              end

   return retv['content'] unless retv['type'] == 'COMMENT'
  end
 end

 def reset
  @ms.pos = 0
 end
# def go_thru
#  @lite.each {|ent| gothru(ent, 1)}
# end


 # All private methods now
 private


 def gothru(entry, indent)
  if entry.class == Array
   puts ' ' * indent + '(' + entry[0]
   entry[1].each {|ent| gothru(ent, indent+2)}
   puts ' ' * indent + ')' if @@debug
  else
   puts ' ' * indent + '~' +entry+'~'
  end
 end

 def group
  puts "OPENING PARENTHESIS Found <#{@ms.match[0]}>" if @@debug
  submatch = []
  word     = []
  while true
   return nil unless md   = @ms.find(@@group)
   return nil unless retv = case md[0]
                             when "(" 
                              submatch << word.join unless word.empty?
                              word.clear
			      group

                             when "{"       then curlyb
                             when @@dquote  then dquote
                             when @@comment then comment
                             when @@space   then space
                             when @@others  then others
                             else
                              puts "CLOSING PARENTHESIS Found <#{md[0]}>" if @@debug
                              submatch << word.join unless word.empty?
                              return Hash['type'    =>'GROUP', 
				          'content' => submatch.length > 1 ? [submatch[0], submatch.slice(1...submatch.length)] :
				                   (!submatch.empty? ? submatch[0] : 'LISPISH_EMPTY_OPENING_CLOSING_PARENTHESIS')
			                 ]
                            end
  
    case retv['type']
     when 'GROUP' then submatch << retv['content']
     when 'SPACE' 
       submatch << word.join unless word.empty?
       word.clear
     when 'COMMENT'
     else word << retv['content']
    end
  end
 end
 
 def comment
  puts "COMMENT found <#{@ms.match[0]}>" if @@debug
  Hash['type'=>'COMMENT', 'content'=>@ms.match[0]]
 end

 def dquote
  puts "DQUOTE found <#{@ms.match[0]}>" if @@debug
  Hash['type'=>'DQUOTE', 'content'=>@ms.match[0].slice(1...@ms.match.end(0)-1)]
 end
 
 def curlyb
  puts "OPEN CBRACE found <#{@ms.match[0]}>" if @@debug
  ipos = @ms.pos
  while true
   return nil unless md = @ms.find(@@curlyb)

    if md[0] == "{"
     curlyb
    else
     puts "CLOSE CBRACE found <#{md[0]}>" if @@debug
     return Hash['type'=>'CBRACE', 'content'=>@ms.string.slice(ipos...@ms.pos-1)]
    end
  end
 end

 def space
  puts "SPACE found" if @@debug
  Hash['type'=>'SPACE']
 end

 def others
  puts "OTHERS found <#{@ms.match[0]}>" if @@debug
  Hash['type'=>'OTHERS', 'content'=>@ms.match[0]]
 end
end

class Mstring
 attr_reader :pos, :match

 def initialize(mstring)
  @post_match = @s = String.new(mstring)
  @pos  = 0
  @match = nil
 end

 def find (re)
  if re.class == Regexp
   @match = re.match(@post_match)
   if @match
    @post_match = @match.post_match 
    @pos        += @match.end(0)
   end

   return @match
  else
   puts "(Lispish::find) -W- Argument is not a REGEXP"
   return nil
  end
 end

 def pos=(pos)
  @post_match = @s.slice(pos...@s.length)
  @pos        = pos
 end

 def string
  @s 
 end
end

class Hutils
 @@debug = false

 def Hutils.debug=(nv)
   @@debug = nv
 end

 def Hutils.read(hfile, *cmap)
  unless File.size?(hfile)
   puts "(HUtils::hread) -E- File #{hfile} " +  (File.exist?(hfile) ? "does not exist" : "is empty")
   return
  end

  cmdmap = cmap.last.class==Hash ? cmap.pop : nil
  filec = Lispish.new(IO.readlines(hfile).join)
  
  lhread = []
  while next_info = filec.next
   puts "######### #{next_info[0]} ############" if @@debug
   lhread.push(*if next_info.class == Array
	         !cmdmap.nil? && cmdmap.has_key?(next_info[0]) ? cmdmap[next_info[0]].call(next_info) : Hutils.to_hash(next_info)
                else 
	         [next_info, 1]
	        end)
	   
  end

  return Hash[*lhread]
 end

 private

 def Hutils.to_hash(topnode)
  arefcount  = topnode[1].find_all {|obj| obj.class == Array}.length
  puts "arefcount: #{topnode[0]} #{arefcount} #{topnode[1].length}" if @@debug
  if arefcount == topnode[1].length
   puts "All Refs                      -> A => {...} : arefcount == topnode[1].length (#{topnode[0]})" if @@debug
   alleaf_a = []
   topnode[1].each {|e| to_hash(e).each {|se| alleaf_a << se}}

   [topnode[0], Hash[*alleaf_a]]
  elsif topnode[1].length == 1
   puts "No Ref => just one scalar     -> A => B : topnode[1].length == 1 (#{topnode[0]}, #{topnode[1][0]})" if @@debug
   puts "=========> <" + [topnode[0], topnode[1][0]] * " " + ">" if @@debug	
   [topnode[0], topnode[1][0]]
  else
    puts "Mix Ref(s) and scalar(s) / All (>1) scalars -> A => [...] : (#{topnode[0]})" if @@debug
   [topnode[0], topnode[1].map {|ent| ent.class == Array ? Hash[*to_hash(ent)] : ent}]
  end
 end
end

