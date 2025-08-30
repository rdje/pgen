class Hash
 def recurse(hpath=[], &code)
  raise "Missing CODE Block" if code.nil?

   each {|k, v|
    hpath.push k

    if v.class == Hash
     v.recurse(hpath, &code)
    else
     code.call(hpath, v)
    end

    hpath.pop
  }

  self
 end

 def recurse!(hpath=[], &code)
  raise "Missing CODE Block" if code.nil?

  each {|k, v|
    hpath.push k

    if v.class == Hash
     v.recurse!(hpath, &code)
    else
     self[k] = code.call(hpath, v)
    end

    hpath.pop
  }

  self
 end

 def hmerge(rhs)
  rhs.recurse {|p, v| set(p.dup, v)}

  self
 end

 def set(p, v)
  top = p.shift
  if has_key?(top)
   if !p.empty?
    if self[top].class == Hash
     self[top].set(p, v)
    else
     self[top] = Hash.new.set(p, v)    
    end
   else
    self[top] = v
   end
  else
   self[top] = p.empty? ? v : Hash.new.set(p, v)
  end

  self
 end
end
