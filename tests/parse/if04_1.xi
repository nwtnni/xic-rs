f() {}

f1() {
  if (true) if (true) f() else f()
  if (true) if (true) f() else if (true) f() else f()
  if (true) if (true) f() else if (true) f() else f() else f()
  if (true) if (true) f() else if (true) f() else f() if (true) f() else f()
}