class B extends A {}

class A {
  foo(): int {
    return 1
  }
}

bar(b: B): int {
  return b.foo()
}
