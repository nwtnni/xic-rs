use io
use conv

main(args:int[][]) {
    n:int = 128;
    nodeSum : int[4*n];
    lazy: int[4*n];
    a: int[n];

    i:int = 0;

    while(i<128) {
        a[i] = i+1;
        i=i+1;
    }

    // fix for initializing arrays
    i =0;
    while(i<4*n) {
        lazy[i] = 0;
        nodeSum[i] = 0;
        i = i+1;
    }

    build(1,0,n,nodeSum,a);


    print(unparseInt(sum(10,99,1,0,128,nodeSum,lazy))); // 4895
    increase(-1,999,1,1,0,128,lazy,nodeSum);
    print(unparseInt(sum(10,99,1,0,128,nodeSum,lazy))); // 4895 + 89 = 4984


}

build(id:int, l:int, r:int, nodeSum:int[],a:int[]) {
    if(r-l<2) {
    nodeSum[id] = a[l];
        return;
    }
    mid:int =(l+r)/2;
    build(id*2,l,mid,nodeSum,a);
    build(id*2+1,mid,r,nodeSum,a);
    nodeSum[id] = nodeSum[id*2] + nodeSum[id*2+1];
}

upd(id:int, l:int, r:int, x:int, lazy:int[], nodeSum:int[]) {
    lazy[id] = lazy[id]+x;
    nodeSum[id] = nodeSum[id]+(r-l)*x;
}

shift(id:int, l:int, r:int, lazy:int[],nodeSum:int[]) {
    mid:int = (l+r)/2;
    upd(id*2,l,mid,lazy[id],lazy,nodeSum);
    upd(id*2+1,mid,r,lazy[id],lazy,nodeSum);
}

increase(x:int, y:int,v:int,id:int, l:int, r:int,lazy:int[],nodeSum:int[]) {
    if(x>=r | y<=l) {return}
    if(x<=l & r<=y) {
        upd(id,l,r,v,lazy,nodeSum)
        return;
    }
    shift(id,l,r,lazy,nodeSum);
    mid:int = (l+r)/2;
    increase(x,y,v,id*2,l,mid,lazy,nodeSum);
    increase(x,y,v,id*2+1,mid,r,lazy,nodeSum);
    nodeSum[id] = nodeSum[id*2] + nodeSum[id*2+1];
}

sum(x:int, y:int, id:int, l:int, r:int,nodeSum:int[],lazy:int[]):int {
    if(x>=r | y<=l) {return 0;}
    if(x<=l & r<=y) {
        return nodeSum[id];
    }
    shift(id,l,r,lazy,nodeSum);

    mid:int = (l+r)/2;
    return sum(x,y,id*2,l,mid,nodeSum,lazy) + sum(x,y,id*2+1,mid,r,nodeSum,lazy);

}
