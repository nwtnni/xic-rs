foo() {
    if (a) foo()
    else if (b) foo()
    else bar()

    if (a) {
        if (a) foo(2)
        if (b)
            if (b)
                if (c) foo(3)
                else foo(4)
            else foo(5)
    }


else if (foo()) bar() else foo()
}