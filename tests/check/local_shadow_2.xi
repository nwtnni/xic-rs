class A {
  x: int
}

class B extends A {
  foo(): int {
    x: bool = true
    return this.x
  }
}
