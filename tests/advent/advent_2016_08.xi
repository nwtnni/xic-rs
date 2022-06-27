use assert
use conv
use io
use string
use vector

INPUT: String = new_string_from_array("rect 1x1\n\
    rotate row y=0 by 6\n\
    rect 1x1\n\
    rotate row y=0 by 3\n\
    rect 1x1\n\
    rotate row y=0 by 5\n\
    rect 1x1\n\
    rotate row y=0 by 4\n\
    rect 2x1\n\
    rotate row y=0 by 5\n\
    rect 2x1\n\
    rotate row y=0 by 2\n\
    rect 1x1\n\
    rotate row y=0 by 5\n\
    rect 4x1\n\
    rotate row y=0 by 2\n\
    rect 1x1\n\
    rotate row y=0 by 3\n\
    rect 1x1\n\
    rotate row y=0 by 3\n\
    rect 1x1\n\
    rotate row y=0 by 2\n\
    rect 1x1\n\
    rotate row y=0 by 6\n\
    rect 4x1\n\
    rotate row y=0 by 4\n\
    rotate column x=0 by 1\n\
    rect 3x1\n\
    rotate row y=0 by 6\n\
    rotate column x=0 by 1\n\
    rect 4x1\n\
    rotate column x=10 by 1\n\
    rotate row y=2 by 16\n\
    rotate row y=0 by 8\n\
    rotate column x=5 by 1\n\
    rotate column x=0 by 1\n\
    rect 7x1\n\
    rotate column x=37 by 1\n\
    rotate column x=21 by 2\n\
    rotate column x=15 by 1\n\
    rotate column x=11 by 2\n\
    rotate row y=2 by 39\n\
    rotate row y=0 by 36\n\
    rotate column x=33 by 2\n\
    rotate column x=32 by 1\n\
    rotate column x=28 by 2\n\
    rotate column x=27 by 1\n\
    rotate column x=25 by 1\n\
    rotate column x=22 by 1\n\
    rotate column x=21 by 2\n\
    rotate column x=20 by 3\n\
    rotate column x=18 by 1\n\
    rotate column x=15 by 2\n\
    rotate column x=12 by 1\n\
    rotate column x=10 by 1\n\
    rotate column x=6 by 2\n\
    rotate column x=5 by 1\n\
    rotate column x=2 by 1\n\
    rotate column x=0 by 1\n\
    rect 35x1\n\
    rotate column x=45 by 1\n\
    rotate row y=1 by 28\n\
    rotate column x=38 by 2\n\
    rotate column x=33 by 1\n\
    rotate column x=28 by 1\n\
    rotate column x=23 by 1\n\
    rotate column x=18 by 1\n\
    rotate column x=13 by 2\n\
    rotate column x=8 by 1\n\
    rotate column x=3 by 1\n\
    rotate row y=3 by 2\n\
    rotate row y=2 by 2\n\
    rotate row y=1 by 5\n\
    rotate row y=0 by 1\n\
    rect 1x5\n\
    rotate column x=43 by 1\n\
    rotate column x=31 by 1\n\
    rotate row y=4 by 35\n\
    rotate row y=3 by 20\n\
    rotate row y=1 by 27\n\
    rotate row y=0 by 20\n\
    rotate column x=17 by 1\n\
    rotate column x=15 by 1\n\
    rotate column x=12 by 1\n\
    rotate column x=11 by 2\n\
    rotate column x=10 by 1\n\
    rotate column x=8 by 1\n\
    rotate column x=7 by 1\n\
    rotate column x=5 by 1\n\
    rotate column x=3 by 2\n\
    rotate column x=2 by 1\n\
    rotate column x=0 by 1\n\
    rect 19x1\n\
    rotate column x=20 by 3\n\
    rotate column x=14 by 1\n\
    rotate column x=9 by 1\n\
    rotate row y=4 by 15\n\
    rotate row y=3 by 13\n\
    rotate row y=2 by 15\n\
    rotate row y=1 by 18\n\
    rotate row y=0 by 15\n\
    rotate column x=13 by 1\n\
    rotate column x=12 by 1\n\
    rotate column x=11 by 3\n\
    rotate column x=10 by 1\n\
    rotate column x=8 by 1\n\
    rotate column x=7 by 1\n\
    rotate column x=6 by 1\n\
    rotate column x=5 by 1\n\
    rotate column x=3 by 2\n\
    rotate column x=2 by 1\n\
    rotate column x=1 by 1\n\
    rotate column x=0 by 1\n\
    rect 14x1\n\
    rotate row y=3 by 47\n\
    rotate column x=19 by 3\n\
    rotate column x=9 by 3\n\
    rotate column x=4 by 3\n\
    rotate row y=5 by 5\n\
    rotate row y=4 by 5\n\
    rotate row y=3 by 8\n\
    rotate row y=1 by 5\n\
    rotate column x=3 by 2\n\
    rotate column x=2 by 3\n\
    rotate column x=1 by 2\n\
    rotate column x=0 by 2\n\
    rect 4x2\n\
    rotate column x=35 by 5\n\
    rotate column x=20 by 3\n\
    rotate column x=10 by 5\n\
    rotate column x=3 by 2\n\
    rotate row y=5 by 20\n\
    rotate row y=3 by 30\n\
    rotate row y=2 by 45\n\
    rotate row y=1 by 30\n\
    rotate column x=48 by 5\n\
    rotate column x=47 by 5\n\
    rotate column x=46 by 3\n\
    rotate column x=45 by 4\n\
    rotate column x=43 by 5\n\
    rotate column x=42 by 5\n\
    rotate column x=41 by 5\n\
    rotate column x=38 by 1\n\
    rotate column x=37 by 5\n\
    rotate column x=36 by 5\n\
    rotate column x=35 by 1\n\
    rotate column x=33 by 1\n\
    rotate column x=32 by 5\n\
    rotate column x=31 by 5\n\
    rotate column x=28 by 5\n\
    rotate column x=27 by 5\n\
    rotate column x=26 by 5\n\
    rotate column x=17 by 5\n\
    rotate column x=16 by 5\n\
    rotate column x=15 by 4\n\
    rotate column x=13 by 1\n\
    rotate column x=12 by 5\n\
    rotate column x=11 by 5\n\
    rotate column x=10 by 1\n\
    rotate column x=8 by 1\n\
    rotate column x=2 by 5\n\
    rotate column x=1 by 5")

WIDTH: int = 50
HEIGHT: int = 6

RECT: String = new_string_from_array("rect")
ROTATE: String = new_string_from_array("rotate")
BY: String = new_string_from_array(" by ")

ON: int[] = "#"
OFF: int[] = "."
EMPTY: int[] = ""

main(args: int[][]) {
    input: Vector::<String> = INPUT.split('\n')

    i: int = 0

    grid: bool[HEIGHT][WIDTH]

    clear(grid)

    while i < input.size() {
        command: Command = parse(input.get(i))
        command.evaluate(grid)
        i = i + 1
    }

    println(unparseInt(sum(grid)))
    debug(grid)
}

clear(grid: bool[][]) {
    i: int = 0
    while i < HEIGHT {
        j: int = 0
        while j < WIDTH {
            grid[i][j] = false
            j = j + 1
        }
        i = i + 1
    }
}

sum(grid: bool[][]): int {
    sum: int = 0
    i: int = 0
    while i < HEIGHT {
        j: int = 0
        while j < WIDTH {
            if grid[i][j] {
                sum = sum + 1
            }
            j = j + 1
        }
        i = i + 1
    }
    return sum
}

debug(grid: bool[][]) {
    i: int = 0
    while i < HEIGHT {
        j: int = 0
        while j < WIDTH {
            if grid[i][j] {
                print(ON)
            } else {
                print(OFF)
            }
            j = j + 1
        }
        println(EMPTY)
        i = i + 1
    }
}

class Command {
    debug() {}
    evaluate(grid: bool[][]) {}
}

parse(command: String): Command {
    if command.starts_with(RECT) {
        splits: Vector::<String> = command
            .slice(RECT.size() + 1, command.size())
            .split('x')

        width: int, success: bool = parseInt(splits.get(0).get_array())
        assert(success)

        height: int, success': bool = parseInt(splits.get(1).get_array())
        assert(success')

        return new Rect.init(width, height)
    } else if command.starts_with(ROTATE) {
        splits: Vector::<String> = command.split('=')

        axis: int = splits.get(0).get(splits.get(0).size() - 1)

        splits': Vector::<String> = splits.get(1).split_string(BY)
        a: int, success: bool = parseInt(splits'.get(0).get_array())
        assert(success)

        b: int, success': bool = parseInt(splits'.get(1).get_array())
        assert(success')

        return new Rotate.init(axis, a, b)
    } else {
        assert(false)
        return null
    }
}

final class Rect extends Command {
    width: int
    height: int

    init(width: int, height: int): Rect {
        this.width = width
        this.height = height
        return this
    }

    evaluate(grid: bool[][]) {
        i: int = 0

        while i < height {
            j: int = 0

            while j < width {
                grid[i][j] = true
                j = j + 1
            }

            i = i + 1
        }
    }

    debug() {
        print("Rect ")
        print(unparseInt(width))
        print("x")
        println(unparseInt(height))
    }
}

final class Rotate extends Command {
    axis: int
    a: int
    b: int

    init(axis: int, a: int, b: int): Rotate {
        this.axis = axis
        this.a = a
        this.b = b
        return this
    }

    evaluate(grid: bool[][]) {
        if axis == 'x' {
            rotate_down(grid, a, b)
        } else if axis == 'y' {
            rotate_right(grid[a], b)
        } else {
            assert(false)
        }
    }

    debug() {
        print("Rotate ")
        print({axis})
        print(" ")
        print(unparseInt(a))
        print("x")
        println(unparseInt(b))
    }
}

rotate_right(array: bool[], rotation: int) {
    assert(rotation >= 0)

    size: int = length(array)
    rotation = rotation % length(array)

    array': bool[rotation]

    // Save rightmost `rotation` elements
    i: int = 0
    while i < rotation {
        array'[i] = array[(size - rotation) + i]
        i = i + 1
    }

    // Shift existing array elements right
    j: int = size - 1
    while j >= rotation {
        array[j] = array[j - rotation]
        j = j - 1
    }

    // Fill in saved elements
    k: int = 0
    while k < rotation {
        array[k] = array'[k]
        k = k + 1
    }
}

rotate_down(array: bool[][], column: int, rotation: int) {
    assert(rotation >= 0)

    size: int = length(array)
    rotation = rotation % length(array)

    array': bool[rotation]

    // Save rightmost `rotation` elements
    i: int = 0
    while i < rotation {
        array'[i] = array[(size - rotation) + i][column]
        i = i + 1
    }

    // Shift existing array elements right
    j: int = size - 1
    while j >= rotation {
        array[j][column] = array[j - rotation][column]
        j = j - 1
    }

    // Fill in saved elements
    k: int = 0
    while k < rotation {
        array[k][column] = array'[k]
        k = k + 1
    }
}
