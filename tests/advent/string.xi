use vector

final class String {
    buffer: Vector::<int>

    size(): int {
        return buffer.size()
    }

    get(index: int): int {
        return buffer.get(index)
    }

    get_array(): int[] {
        return slice_array(0, buffer.size())
    }

    push(character: int) {
        buffer.push(character)
    }

    pop(): int {
        return buffer.pop()
    }

    starts_with(substring: String): bool {
        if buffer.size() < substring.size() {
            return false
        }

        i: int = 0
        while i < substring.size() {
            if buffer.get(i) != substring.get(i) {
                return false
            }
            i = i + 1
        }

        return true
    }

    contains(substring: String): bool {
        i: int = 0

        while i + substring.size() < size() {
            j: int = 0

            while j < substring.size() {
                if get(i + j) != substring.get(j) {
                    break
                }

                j = j + 1
            }

            if j == substring.size() {
                return true
            }

            i = i + 1
        }

        return false
    }

    split(character: int): Vector::<String> {
        splits: Vector::<String> = new_vector::<String>()

        i: int = 0
        j: int = 0

        while j < buffer.size() {
            if buffer.get(j) == character {
                if i == 0 {
                    splits.push(slice(i, j))
                } else {
                    splits.push(slice(i + 1, j))
                }

                i = j
            }

            j = j + 1
        }

        if i == 0 {
            splits.push(slice(i, buffer.size()))
        } else {
            splits.push(slice(i + 1, buffer.size()))
        }

        return splits
    }

    split_string(substring: String): Vector::<String> {
        splits: Vector::<String> = new_vector::<String>()

        i: int = 0
        j: int = 0

        while j + substring.size() < buffer.size() {
            if buffer.get(j) == substring.get(0) {
                cut: bool = true

                k: int = 0
                while k < substring.size() {
                    if buffer.get(j + k) != substring.get(k) {
                        cut = false
                        break
                    }
                    k = k + 1
                }

                if cut {
                    if i == 0 {
                        splits.push(slice(i, j))
                    } else {
                        splits.push(slice(i + 1, j))
                    }

                    j = j + substring.size()
                    i = j
                } else {
                    j = j + 1
                }
            } else {
                j = j + 1
            }
        }

        if i == 0 {
            splits.push(slice(i + 1, buffer.size()))
        } else {
            splits.push(slice(i, buffer.size()))
        }

        return splits
    }

    slice(low: int, high: int): String {
        return new_string_from_vector(buffer.slice(low, high))
    }

    slice_array(low: int, high: int): int[] {
        return buffer.slice_array(low, high)
    }
}

new_string(): String {
    string: String = new String
    string.buffer = new_vector::<int>()
    return string
}

new_string_from_array(buffer: int[]): String {
    return new_string_from_vector(new_vector_from_array::<int>(buffer))
}

new_string_from_vector(buffer: Vector::<int>): String {
    string: String = new String
    string.buffer = buffer
    return string
}
