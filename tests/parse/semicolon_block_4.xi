foo() {
    {
        x:int = 5;
        bar();
        x = 5;
        x = {};
    };
}