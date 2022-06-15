final class A {}

template class B<T> extends A {}

foo() {
    b: B::<int> = new B::<int>
}
