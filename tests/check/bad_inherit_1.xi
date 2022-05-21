class A extends B {}

class B {
  foo(): int { return 1 }
}

bar(a: A): int {
  return a.baz()
}
