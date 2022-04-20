foo() {
    while(false) {
        x:int = 5;
        x = bar();
        foo2();
        x = 5;
    }
}