class A {
  a: int
}

class B extends A {
  b: int
}

class C extends B {
  c: int

  foo(): int {
    return a + b + c
  }
}
