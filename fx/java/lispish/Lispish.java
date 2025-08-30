package lispish;

import java.io.*;
import java.util.regex.*;
import java.util.*;
import edu.umd.cs.piccolo.nodes.*;

public class Lispish {
 String string;
 Matcher groupname;
 Matcher matcher;

 Pattern top;
 Pattern group;
 Pattern dquotes;
 Pattern curlyb;

 // Lispish constructor
 Lispish (String filename) {
  string              = slurp(filename);

  groupname          = Pattern.compile("(\\S+)").matcher(string);

  String  top_re      = "\\(|\\)";
  String  group_re    = "\\(|\\{|\".*?(?<!\\\\)\"|\\s+|[^\\s\"\\{\\}\\(\\)]+|\\)";
  String  curlyb_re   = "\\{|\\}";

  top                 = Pattern.compile(top_re);
  group               = Pattern.compile(group_re);
  curlyb              = Pattern.compile(curlyb_re);

  // Initial matcher make use of 'top'
  matcher             = top.matcher(string);
 }

 // Entry point of the Lisp-like file parser
 public Object get () {return top_m();}

 private Object top_m () {

  Object retv;
  ArrayList<Object> group_set = new ArrayList<Object>();
  while(true) {
   matcher.usePattern(top);

   if(matcher.find()) {
    if(matcher.group().equals("(")) {
     retv = group_m();
     if(retv == null) return null;

     group_set.add(retv);

     System.out.println("top: ################################################\n");
    } else {
     System.out.println("\n(Lispish) -E- Syntax Error\n");
     return null;
    }

   } else return group_set.isEmpty() ? null : group_set.toArray();

  }
 }

 private Object group_m () {
  int ipos = matcher.end();

  System.out.println("\ngroup: Opening Parenthesis Found");

  groupname.reset().find(matcher.end());
  String lgroupname = groupname.group();
  System.out.println("group: NAME ["+lgroupname+"]");
  
  
  ArrayList<Object> submatchs = new ArrayList<Object>();
  StringBuffer word = new StringBuffer();
  Object retv;
  String type = "";
  while(true) {
   if(matcher.usePattern(group).find()) {

    if(matcher.group().equals("("))   {
     System.out.println("Storing word in 'submatchs' due to OPENING Parenthesis\n");
     if (word.length() != 0) submatchs.add(word.toString()); 
     word.setLength(0);

     type = "group_m";   
     retv = group_m();   
    }

    else if (matcher.group().equals("{"))                                                 {type = "curlyb_m";  retv = curlyb_m();  }
    else if (Pattern.compile("^\"").matcher(matcher.group()).find())                      {type = "dquotes_m"; retv = dquotes_m(); }
    else if (Pattern.compile("^\\s+").matcher(matcher.group()).find())                    {type = "spaces_m";  retv = spaces_m();  }
    else if (Pattern.compile("[^\\s\"\\{\\}\\(\\)]+").matcher(matcher.group()).find())    {type = "rest_m";    retv = rest_m();    }
    else {
     //System.out.println("group: Closing Parenthesis Found for GROUP("+lgroupname+")["+string.substring(ipos, matcher.end()-matcher.group().length())+"] UP\n");
     System.out.println("Storing word in 'submatchs' due to CLOSING Parenthesis\n");
     if (word.length() != 0) submatchs.add(word.toString()); 
     word.setLength(0);
     
     if(submatchs.size() > 1) {
      // Object tmp = submatchs.remove(0);
      // retsub     = submatchs.toArray(); 
      Object[] oretv = {submatchs.remove(0), submatchs.toArray()};
      return oretv;
     } else return submatchs.size() != 0 ? submatchs.get(0) : "LISPISH_EMPTY_OPENING_CLOSING_PARENTHESIS";
     
    }

    if(retv == null) return null;

    //System.out.println("CLASS["+retv.getClass().getSimpleName()+"]");
    if (!type.equals("group_m")) {
     if (!type.equals("spaces_m")) {word.append(retv.toString()); System.out.println("pushed into word:["+word.toString()+"]\n");}
     else if (word.length() != 0) {
      System.out.println("Storing word in 'submatchs' due to SPACE\n");
      if (word.length() != 0) submatchs.add(word.toString()); 
      word.setLength(0);
     }
    } else submatchs.add(retv);

   } else {
     System.out.println("group: NOOOOOOOO Closing Parenthesis Found");
     return null;
   }
  }
 }

 private String curlyb_m () {
  int ipos = matcher.end();

  System.out.println("curlyb_m: Opening Brace Found\n");

  while(true) {
   if(matcher.usePattern(curlyb).find()) {

    if (matcher.group().equals("{")) curlyb_m();
    else {
      System.out.println("curlyb_m: Closing Brace Found["+string.substring(ipos, matcher.end())+"]\n");
      return string.substring(ipos, matcher.end());
    }

   } else {
     System.out.println("curlyb_m: --NO-- Closing Brace Found");
     return null;
   }
  }
 }

 private String dquotes_m () {System.out.println("dquotes_m: DQUOTE Found["+matcher.group().replaceAll("^\"|\"$", "")+"]"); return matcher.group().replaceAll("^\"|\"$", "");}
 private String spaces_m  () {System.out.println("spaces_m: SPACE Found["+matcher.group()+"]"); return matcher.group();}
 private String rest_m    () {System.out.println("rest_m: REST Found["+matcher.group()+"]"); return matcher.group();}

 static void propagate_thru (Object objstr) {
  System.out.println("Current level NAME["+((Object[])objstr)[0]+"]");
  Object[] carray = (Object[])((Object[])objstr)[1];
  for (Object cobj: carray) {
   if(cobj.getClass().getSimpleName().equals("String")) {
     System.out.println("STRING VALUE["+cobj+"]");
   } else {
     System.out.println("There one level down NAME["+((Object[])cobj)[0]+"] GERONIMOOOOOOOOOOOOOOOOOOOOOOoooooooooooooooooo");
     propagate_thru(cobj);
   }
  }
 }

 // Slurp an entire file into a String object
 String slurp (String filename) {
  File filepath = new File(filename);

  String stringified;
  try {
   FileReader filereader = new FileReader(filepath);
   if (filepath.canRead()) {
    int filesz = (int)filepath.length();
    char[] allfile = new char[filesz];
    
    filereader.read(allfile, 0, filesz);
    stringified = new String(allfile);
   } else {
   stringified = null;
   }
  } catch (Throwable f) {
   System.out.println(f.toString());
   stringified = null;
  }

  return stringified.replaceAll(";.*", "");
 }
 


 public static void main (String[] argv) {
   //Lispish mytest = new Lispish("../conftest.txt");
   Lispish mytest = new Lispish("/home/rdje/conf/stan.conf");
   // System.out.println(mytest.string);

   Object retv = mytest.get();
   for (Object cobj: (Object[])retv) propagate_thru(cobj);
 }
 
}


