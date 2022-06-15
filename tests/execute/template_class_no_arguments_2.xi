use io
use conv

main(args: int[][]) {

    a: A::<> = new A::<>
    a.value = 1

    print(a.method())
    print(", ")
    println(unparseInt(a.value))
}
