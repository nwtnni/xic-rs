f() {}

f1() {
  if (true) f()
  if (false) f()
  if (g()) f()
  if ((true)) f()
  if ((g())) f()
}

f2(b:bool) {
  if (b) f()
}

f3(b1:bool, b2:bool) {
  if (b1 & b2) f()
  if (b1 | b2) f()
}

g(): bool {
  return true
}
