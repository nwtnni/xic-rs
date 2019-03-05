foo(): int { return foo() }

bar() {
  i: int[foo()]
  b: int["invalid"]
}
