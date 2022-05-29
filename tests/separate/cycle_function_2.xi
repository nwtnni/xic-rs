use cycle_function_1

odd(i: int): bool {
    if i == 0 {
        return false
    } else {
        return even(i - 1)
    }
}
