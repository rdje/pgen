#! python

def myfunc (*pa, **opt) :
 if opt.has_key('assign') and opt["assign"] != None : toto = pa[0]
 else                                               : toto = "---Nothing---"

 # Like Perl's Closure, but named
 def hello () : print "foo<"+ toto +">"
 
 return hello
 
# Like eval in Perl
exec 'print "damn"'

code_obj = compile ("1", '<string>', 'eval')

print "NAME<" + str(code_obj.__class__) +">"
def avv_hset (h, kl, v) :
  if not isinstance(h, dict) : return
  if kl == []                : return

  k = kl.pop(0)
  if  h.has_key(k): 
    if   kl == []               : h[k] = v
    elif isinstance(h[k], dict) : avv_hset (h[k], kl, v)
    else : print "avv_hset: Error";
  else :
    if   kl == [] : h[k] = v
    else          :
      h[k] = {}
      avv_hset (h[k], kl, v)

def recurse (h, c, kl=None) :
  if not isinstance (h, dict) : return 
  if not callable   (c)       : return

  if kl == None : kl = []
  for k in h.keys() :
    kl.append(k)
    print k
    nxkl = [v for v in kl]
    if isinstance(h[k], dict) : recurse (h[k], c, nxkl)
    else : c (nxkl, h[k])

    kl.pop
      


h = {}
avv_hset(h, ['a', 'b', 'c', "how-are-you", "not-so-bad"], "yahoo-yahoo")
avv_hset(h, ['a', 'b', 'd', "not-so-bad", "how-are-you"], "google-google")

print "test:1<"+ h['a']['b']['c']["how-are-you"]["not-so-bad"] +">"
print "test:2<"+ h['a']['b']['d']["not-so-bad"]["how-are-you"] +">"

def  foo (x, y) : 
	print  "<"+str(x)+"> <"+y+">"
recurse (h, foo)
