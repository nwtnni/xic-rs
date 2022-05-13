use io
use conv

// head -n 21 tests/execute/cse3.xi \
//  | tail -n 10 \
//  | tr -d ' ' \
//  | awk 'BEGIN { FS = ":" } { print $1 " + " }' \
//  | shuf \
//  | tr -d '\n'
//  | xclip -selection clipboard -i
main(args:int[][]) {
    t0: int = 0
    t1: int = 1
    t2: int = 2
    t3: int = 3
    t4: int = 4
    t5: int = 5
    t6: int = 6
    t7: int = 7
    t8: int = 8
    t9: int = 9

    a: int = t7 + t8 + t3 + t1 + t4 + t9 + t2 + t6 + t5 + t0;
    b: int = t5 + t1 + t7 + t4 + t3 + t8 + t0 + t2 + t6 + t9;
    c: int = t2 + t6 + t7 + t1 + t5 + t0 + t9 + t8 + t3 + t4;
    d: int = t2 + t9 + t0 + t3 + t4 + t8 + t1 + t7 + t5 + t6;
    e: int = t9 + t1 + t6 + t3 + t5 + t4 + t7 + t8 + t2 + t0;
    f: int = t1 + t4 + t8 + t5 + t3 + t0 + t7 + t6 + t2 + t9;
    g: int = t7 + t3 + t9 + t0 + t1 + t8 + t4 + t2 + t6 + t5;
    h: int = t9 + t0 + t2 + t4 + t1 + t7 + t3 + t8 + t6 + t5;
    i: int = t1 + t0 + t6 + t2 + t7 + t9 + t3 + t5 + t4 + t8;
    j: int = t7 + t3 + t1 + t2 + t8 + t4 + t6 + t5 + t0 + t9;
    k: int = t6 + t9 + t1 + t5 + t2 + t3 + t4 + t7 + t8 + t0;
    l: int = t5 + t0 + t8 + t9 + t6 + t4 + t7 + t3 + t1 + t2;
    m: int = t0 + t4 + t2 + t9 + t5 + t3 + t6 + t8 + t1 + t7;
    n: int = t1 + t3 + t0 + t9 + t4 + t5 + t7 + t8 + t2 + t6;
    o: int = t4 + t2 + t0 + t3 + t1 + t6 + t5 + t9 + t7 + t8;
    p: int = t8 + t4 + t7 + t9 + t0 + t1 + t3 + t6 + t5 + t2;
    q: int = t6 + t3 + t0 + t7 + t5 + t9 + t2 + t4 + t8 + t1;
    r: int = t0 + t8 + t2 + t9 + t6 + t5 + t4 + t3 + t1 + t7;
    s: int = t4 + t2 + t3 + t7 + t5 + t9 + t6 + t8 + t1 + t0;
    t: int = t2 + t5 + t4 + t9 + t1 + t3 + t6 + t0 + t7 + t8;
    u: int = t0 + t8 + t2 + t1 + t9 + t6 + t3 + t4 + t7 + t5;
    v: int = t9 + t1 + t5 + t7 + t6 + t4 + t8 + t0 + t2 + t3;
    w: int = t5 + t4 + t9 + t1 + t3 + t6 + t2 + t8 + t0 + t7;
    x: int = t3 + t8 + t9 + t6 + t5 + t7 + t1 + t4 + t2 + t0;
    y: int = t8 + t1 + t4 + t5 + t3 + t7 + t0 + t9 + t6 + t2;
    z: int = t3 + t2 + t6 + t1 + t7 + t4 + t5 + t0 + t8 + t9;

    println(unparseInt(a))
    println(unparseInt(b))
    println(unparseInt(c))
    println(unparseInt(d))
    println(unparseInt(e))
    println(unparseInt(f))
    println(unparseInt(g))
    println(unparseInt(h))
    println(unparseInt(i))
    println(unparseInt(j))
    println(unparseInt(k))
    println(unparseInt(l))
    println(unparseInt(m))
    println(unparseInt(n))
    println(unparseInt(o))
    println(unparseInt(p))
    println(unparseInt(q))
    println(unparseInt(r))
    println(unparseInt(s))
    println(unparseInt(t))
    println(unparseInt(u))
    println(unparseInt(v))
    println(unparseInt(w))
    println(unparseInt(x))
    println(unparseInt(y))
    println(unparseInt(z))
}
