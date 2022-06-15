template final class A<T> {}

template class B<T> extends A::<T> {}

foo() {
    b: B::<int> = new B::<int>
}
