lib_file::  -> group    .push
LX          {return \@lib_file}

group: /\b(\w+)\s*\(\s*((?s:.*?))\s*\)\s*\{/ /\}/  I {
 my ($grouptype, $groupname) = @IMATCH_LIST;
                 $groupname  =~ s/"//go if $groupname;
}

-> group                 .push
-> cattribute            .push
-> sattribute            .push
-> group[1]              {
 return  ['GROUP', $grouptype, $groupname, \@group]
}

LX {say "GROUP <$grouptype>($groupname) Has a syntax error."; exit 1}

sattribute: /\b(\w+)\s*:\s*(.*?)\s*;/    I {
 my ($attribute_name, $value) = @IMATCH_LIST;
 $value =~ s/"//go if $value;

 return ['SATTRIBUTE', $attribute_name, $value]
}

cattribute: /\b(\w+)\b\s*\(((?s:.)*?)\)\s*;/ I {
 my ($attribute_name, $value) = @IMATCH_LIST;
 $value               =~ s/"|\\|\s//go if $value;

 return ['CATTRIBUTE', $attribute_name, [split /,/o, $value]]
}
