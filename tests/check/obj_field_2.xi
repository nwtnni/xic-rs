class A {
  x: int
}

class B extends A {
  foo(): int {
    x = 3
    return x
  }
}
