class A {
  foo(): int { return 1 }
}

class B extends A {
  foo(): int { return 2 }
}

class C extends A {
  foo(): int { return 3 }
}

main(): A[] {
  return {new B, new A, new C}
}
