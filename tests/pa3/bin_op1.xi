arrays() {
    b:bool = {1} == {}
    a:int[] = {1, 2}
    a2:int[] = {}
    b = a == {2,2}
    b = a == a2
    a3:int[] = {1} + {3}
}

bools() {
    if (true == false) 
    if (false != true)
    if (true & true)
    if (false | false) {}
}

ints() {
    i:int = 1 + 2 - 3 * 4 *>> 5 / 6 % 7
    b:bool = 1 < 2 | 3 > 4 | 5 <= 6 | 7 >= 8 | 9 == 10 | 11 != 12
}