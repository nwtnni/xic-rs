foo() {
    x:int = length({})
    x = length({1})
    x = length({2})

    x = length({true})
    x = length({false})

    x = length({1,2,3,4,5,6,7,8})
    x = length({true, true, true, false})

    x = length({{1},{2}})
    x = length ({{false},{true}})

    x = length({{1},{}})
    x = length({{false},{}})

    x = length({{},{}})
    x = length({{{}},{{}}})
}
