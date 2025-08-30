#! env groovy

// a List of Map
list = [[a:1, a1:11], [b:2, b1:21], [c:3, c1:31]]

println "0: list-of-map <$list>"

// If I got it right spreading all inner Maps of the List should yield a Map
// [a:1, a1:11, b:2, b1:21, c:3, c1:31]
map   = [list.collect  {it.toSpreadMap()}.spread()]

// Doesn't work as expected
println "0: expecting a map <$map>"

// Map spreading doesn't seems to be working
// I may be missing something !

// List of Map
println "1: list-of-map <${[[a:1, b:2, c:3]]}>"

// Should be a Map after spreading the inner map.
// Does not work, pls help
// It looks like a SpreadMap enclosed in '[ ... ]' is not equivalent to a Map, why ?
println "2: expecting a map <${[[a:1, b:2, c:3].spread()]}>"


// List spreading works as expected
list1  = [1, 2, 3, 4, 5, 6]
// A SpreadList enclosed in '[ ... ]' is equivalent to a List, that make sense.
list2  = [*list1]
list21 = [list1.spread()]

list3  = [list1]

println "list2-is-a-list-of-numbers  <$list2>"
println "list21-is-a-list-of-numbers <$list21>"
println "list3-is-a-list-of-lists    <$list3>"

