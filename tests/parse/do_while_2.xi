f(b: bool) {
  do if true f() while b
}

f2(b:bool) {
  do if true if false f2(b) else if true f(b) else f2(b) while b
}
