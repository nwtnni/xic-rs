use assert
use io
use conv

INPUT: int[] = "RLRDDRLLDLRLUDDULLDRUUULDDLRLUDDDLDRRDUDDDLLURDDDLDDDRDURUDRDRRULUUDUDDRRRLRRRRRLRULRLLRULDRUUDRLRRURDDRLRULDLDULLLRULURRUULLRLLDDDDLLDURRUDLDLURDRDRDLUUUDDRDUUDDULLUURRDRLDDULURRRUDLLULULDLLURURUDRRRRUDRLRDLRRLDDRDDLULDLLLURURDUDRRRRUULURLRDULDRLUDRRUDDUULDURUDLDDURRRDLULLUUDRLLDUUDLDRUDDRLLLLLLDUDUDDLRDLRRDRUDDRRRLLRRDLLRLDDURUURRRDDLDUULLDLDLRURDLLLDDRUUDRUDDDDULRLLDUULRUULLLULURRRLLULDLDUDLDLURUDUDULLDLLUUDRRDRLUURURURURDLURUUDLDRLUDDUUDULDULULLLDLDDULLULLDULRRDRULLURRRULLDDDULULURLRDURLLURUDDULLRUDLRURURRDRDUULDRUUDURDURDDLRDUUULDUUDRDURURDRRRURLLDDLLLURURULULUDLRDLDRDRURLRLULRDLU\n\
UDLDURRULDRDDLDUULUDLDUULUURDDRUDRURRRUDRURLLDDRURLDLRDUUURDLLULURDDUDDDRRRURLLDLDLULRDULRLULDLUUDLLRLDLRUUULDDUURDLDDRRDLURLDUDDRURDRRURDURRRLUULURDDLRDLDRRRLDUDRLRLLRLDDUULDURUUULLLRRRRRRRDRRRDRLUULDLDDLULDRDUDLLUDRRUDRUUDULRLUURDDDDRRUUDLURULLLURDULUURDRDDURULRUDRRDLRDUUUUUDDDRDRDDRUDRDDDRLRUUDRDRDDDLUDRDRLDRDDRULURDRLDRUDUDRUULRLLUDRDRLLLLDUDRRLLURDLLLDRRUDDUDRLRLDUDRLURRUUULURDDRUURRLDRLRRRUUDLULDDDRDLDUUURLLUULDDRRUDLDDRUDUDUURURDDRDULLLLLULRRRDLRRRDDDLURDDDDLUULLLRDDURRRRLURRLDDLRUULULRDRDDDDLDUUUUUUDRRULUUUDD\n\
UURDRRUDLURRDDDLUDLRDURUDURDLLLLRDLRLRDDRDRDUUULRDLLDLULULRDUDDRRUUDURULDLUDLRDRUDLDDULLLDDRDLLDULLLURLLRDDLDRDULRRDDULRDURLLRUDRLRRLUDURLDRDLDLRLLLURLRRURDLDURDLUDULRDULLLDRDDRDLDRDULUULURDRRRLDRRUULULLDDRRLDLRUURLRUURLURRLLULUUULRLLDDUDDLRLDUURURUDLRDLURRLLURUDLDLLUDDUULUUUDDDURDLRRDDDLDRUDRLRURUUDULDDLUUDDULLDDRRDDRRRUDUDUDLDLURLDRDLLLLDURDURLRLLLUUDLRRRRUDUDDLDLRUURRLRRLUURRLUDUDRRRRRRRLDUDDRUDDLUDLRDDDRLDUULDRDRRDLDRURDLDRULRLRLUDRDLRRUURUUUUDLDUUULLLRRRRRDLRRURDDLLLLUULDLLRULLUDLLDLLUDLRLRRLRURDDRRL\n\
URDRDLLRDDDLLLDDLURLRURUURRRLUURURDURRLLUDURRLRLDLUURDLULRRDRUDDLULDLDRLDLRLRRLLLDDDUDDDLRURURRLLDRRRURUDLRDDLLDULDDLDRLUUUDRRRULDUULRDDDLRRLLURDDURLULRDUDURRLLDLLRLDUDDRRDDLRLLLDUDRLUURRLLDULRLDLUUUUUDULUDLULUDDUURRURLDLDRRLDLRRUDUDRRDLDUDDLULLDLLRDRURDRDRRLDDDDRDDRLLDDDLLUDRURLURDRRRRRUDDDUDUDDRDUUDRRUDUDRLULDDURULUURUUUURDRULRLRULLDDRRRUULRRRRURUDLDLRDLLDRLURLRUULLURDUDULRRURLRLLRRLLLURULRRRLDDUULLUUULRRDRULUUUUDRDRRDLRURLRLLRLRRRDRDRLDLUURUURULLDLULRRLRRDRULRRLLLDDURULLDLDLDLUUURDLDLUUDULRLLUDDRRDLLDLDLDURLUURRDDRRURDRLUDRLUUUDLDULDLUDRLDUDDLLRUDULLLLLDRRLLUULLUUURRDDUURDLLRDDLRLLU\n\
LDUDRRDLUUDDRLLUUULURLDUDLUDLRLDRURLULRLLDDLRRUUUDDDDRDULDDUUDLRUULDRULLRDRUDDURLDUUURRUDUDRDRDURRDLURRRDRLDLRRRLLLRLURUURRDLLRDLDDLLRDUDDRDUULRULRRURLUDDUDDDUULLUURDULDULLLLRUUUDDRRRLDDDLDLRRDRDRDLUULRLULDRULDLRDRRUDULUDLLUDUULRDLRRUUDDLLDUDDRULURRLULDLDRRULDDRUUDDLURDLRDRLULRRLURRULDUURDLUDLLDRLDULLULDLLRDRDLLLUDLRULLRLDRDDDLDDDLRULDLULLRUUURRLLDUURRLRLDUUULDUURDURRULULRUUURULLLRULLURDDLDRLLRDULLUDLDRRRLLLLDUULRRLDURDURDULULDUURLDUDRLRURRDLUUULURRUDRUUUDRUR"

main(args: int[][]) {

    lines: int[][] = split(INPUT, '\n')

    i: int = 0

    while i < length(lines) {

        x: int = -2
        y: int = 0

        j: int = 0

        while j < length(lines[i]) {
            if lines[i][j] == 'U' {
                y = clamp(y, -1, x)
            } else if lines[i][j] == 'D' {
                y = clamp(y, 1, x)
            } else if lines[i][j] == 'L' {
                x = clamp(x, -1, y)
            } else if lines[i][j] == 'R' {
                x = clamp(x, 1, y)
            } else {
                assert(false)
            }

            j = j + 1
        }

        // :(
        if y == -2 & x == 0 {
            print(unparseInt(1))
        } else if y == -1 & x == -1 {
            print(unparseInt(2))
        } else if y == -1 & x == 0 {
            print(unparseInt(3))
        } else if y == -1 & x == 1 {
            print(unparseInt(4))
        } else if y == 0 & x == -2 {
            print(unparseInt(5))
        } else if y == 0 & x == -1 {
            print(unparseInt(6))
        } else if y == 0 & x == 0 {
            print(unparseInt(7))
        } else if y == 0 & x == 1 {
            print(unparseInt(8))
        } else if y == 0 & x == 2 {
            print(unparseInt(9))
        } else if y == 1 & x == -1 {
            print("A")
        } else if y == 1 & x == 0 {
            print("B")
        } else if y == 1 & x == 1 {
            print("C")
        } else if y == 2 & x == 0 {
            print("D")
        } else {
            assert(false)
        }

        i = i + 1
    }

    println("")
}

clamp(a: int, delta: int, b: int): int {
    if abs(a + delta) + abs(b) > 2 {
        return a
    } else {
        return a + delta
    }
}

abs(a: int): int {
    if a < 0 {
        return -a
    } else {
        return a
    }
}

split(string: int[], character: int): int[][] {
    count: int = 1

    // First pass: compute number of splits
    i: int = 0
    while i < length(string) {
        if string[i] == character {
            count = count + 1
        }

        i = i + 1
    }

    // Second pass: compute split indices
    indices: int[count + 1]
    indices[0] = 0
    indices[count] = length(string)

    i = 0
    j: int = 1
    while i < length(string) {
        if string[i] == character {
            indices[j] = i
            j = j + 1
        }
        i = i + 1
    }

    // Third pass: compute splits
    splits: int[count][]

    i = 0

    while i + 1 < length(indices) {
        low: int = indices[i]
        high: int = indices[i + 1]

        // Skip split character for subsequent splits
        if i > 0 {
            low = low + 1
        }

        split': int[high - low]
        j = 0

        while j < length(split') {
            split'[j] = string[low + j]
            j = j + 1
        }

        splits[i] = split'
        i = i + 1
    }

    return splits
}
