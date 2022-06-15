clamp(a: int, low: int, high: int): int {
    return max(min(a, high), low)
}

max(a: int, b: int): int {
    if a > b {
        return a
    } else {
        return b
    }
}

min(a: int, b: int): int {
    if a < b {
        return a
    } else {
        return b
    }
}

abs(a: int): int {
    if a < 0 {
        return -a
    } else {
        return a
    }
}
