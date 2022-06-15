use vector

class String {
    buffer: Vector::<int>

    size(): int {
        return buffer.size()
    }

    get(index: int): int {
        return buffer.get(index)
    }

    getArray(): int[] {
        return sliceArray(0, buffer.size())
    }

    push(character: int) {
        buffer.push(character)
    }

    pop(): int {
        return buffer.pop()
    }

    split(character: int): Vector::<String> {
        splits: Vector::<String> = newVector::<String>()

        i: int = 0
        j: int = 0

        while j < buffer.size() {
            if buffer.get(j) == character {
                if i > 0 {
                    splits.push(slice(i + 1, j))
                } else {
                    splits.push(slice(i, j))
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

    slice(low: int, high: int): String {
        return newStringFromArray(sliceArray(low, high))
    }

    sliceArray(low: int, high: int): int[] {
        buffer': int[high - low]
        i: int = low
        while i < high {
            buffer'[i - low] = buffer.get(i)
            i = i + 1
        }
        return buffer'
    }
}

newString(): String {
    string: String = new String
    string.buffer = newVector::<int>()
    return string
}

newStringFromArray(buffer: int[]): String {
    return newStringFromVector(newVectorFromArray::<int>(buffer))
}

newStringFromVector(buffer: Vector::<int>): String {
    string: String = new String
    string.buffer = buffer
    return string
}
