decl-version 2.0
input-language rust
var-comparability implicit

ppt /home/olegian/DATIR/decls-gen/tests/simple/main.bar:::ENTER
ppt-type enter
variable p1[..].length
  var-kind variable
  dec-type usize
  rep-type int
  comparability -1
variable p1[..][..]
  var-kind array
  dec-type u32
  rep-type int
  array 1
  enclosing-var p1[..]
  comparability -1
variable p1.length
  var-kind variable
  dec-type usize
  rep-type int
  comparability -1

ppt /home/olegian/DATIR/decls-gen/tests/simple/main.main:::ENTER
ppt-type enter

ppt /home/olegian/DATIR/decls-gen/tests/simple/main.foo:::ENTER
ppt-type enter
variable p1::V1.0
  var-kind field 0
  dec-type u32
  rep-type int
  enclosing-var p1::V1
  comparability -1
variable p2.0
  var-kind field 0
  dec-type u32
  rep-type int
  enclosing-var p2
  comparability -1
variable p1::V1.1
  var-kind field 1
  dec-type f64
  rep-type double
  enclosing-var p1::V1
  comparability -1
variable p4.a
  var-kind field a
  dec-type u32
  rep-type int
  enclosing-var p4
  comparability -1
variable p4.c
  var-kind field c
  dec-type i32
  rep-type int
  enclosing-var p4
  comparability -1
variable p4.b
  var-kind field b
  dec-type f64
  rep-type double
  enclosing-var p4
  comparability -1

