class C extends B {
  foo(): int { return 0 }
}

class B extends A {
  bar(): int { return 1 }  
}

class A {
  baz(): bool { return false }
}

two(c: C) : bool {
  x: int = c.foo()
  x = c.bar()
  return c.baz()
}


