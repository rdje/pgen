# --------------------------------------------
#  Parses the output of 
#
#  dssc vhistory -all -report verbose
# --------------------------------------------

vhistory::  I {my (@vhistory, @capt, @object_hier, $cur_object)}
LX  {
 if (@capt) {
   push @object_hier, [$capt[0][0] eq '?branch:' ? '?branch_entry:' : '?version_entry:', [@capt]];
 }

 if (@object_hier) {
  push @$cur_object, [@object_hier];
  push @vhistory,    $cur_object;
 }

 return ['?ds_vhistory:', \@vhistory]
} 

-> object             {
  if (@capt) {
    push @object_hier, [$capt[0][0] eq '?branch:' ? '?branch_entry:' : '?version_entry:', [@capt]];
    @capt = ();
  }

  if (@object_hier) {
   push @$cur_object, [@object_hier];
   push @vhistory,    $cur_object;

   @object_hier = ();
  }

  $cur_object  = call(object);
  print "\tObject   $cur_object->[1]\n";
}


-> separator          {
  if (@capt) {
    push @object_hier, [$capt[0][0] eq '?branch:' ? '?branch_entry:' : '?version_entry:', [@capt]];
    @capt = ();
  }
}


-> branch             {push @capt, call(branch)}
-> version            {push @capt, call(version)}
-> branch_tags        {push @capt, call(branch_tags)}
-> version_tags       {push @capt, call(version_tags)}
-> date               {push @capt, call(date)}
-> author             {push @capt, call(author)}
-> comment            {push @capt, call(comment)}
-> manifest           {push @capt, call(manifest)}
-> derived_from       {push @capt, call(derived_from)}



separator:    /-{20,}/
object:       /(?i)\nobject:\s+(\S+)/                                             I {return  ['?object:',       @IMATCH_LIST]}
branch:       /(?i)\nbranch:\s+(\S+)/                                             I {return  ['?branch:',       @IMATCH_LIST]}
branch_tags:  /(?is)\nbranch\s+tags:\s+(?:([^:]+?),\s*(.+?)\s*,\s*(\w+)|(\S+))/   I {return  ['?branch_tags:',  @IMATCH_LIST]}
version_tags: /(?is)\nversion\s+tags:\s+(?:([^:]+?),\s*(.+?)\s*,\s*(\w+)|(\S+))/  I {return  ['?version_tags:', @IMATCH_LIST]}
version:      /(?i)\nversion:\s+(\S+)/                                            I {return  ['?version:',      @IMATCH_LIST]}
date:         /(?i)\ndate:\s+(.+)/                                                I {return  ['?date:',         @IMATCH_LIST]}
comment:      /(?i)\ncomment:\s+(.+)/                                             I {return  ['?comment:',      @IMATCH_LIST]}
author:       /(?i)\nauthor:\s+(.+)/                                              I {return  ['?author:',       @IMATCH_LIST]}
derived_from: /(?i)\nderived_from:\s+(\S+)/                                       I {return  ['?derived_from:', @IMATCH_LIST]}
manifest:     /(?is)\nmanifest:\s+.+?\n\n/                                        I {return  ['?manifest:']}
