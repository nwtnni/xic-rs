// if statements

foo(a:int, b:bool):int[], int {
	c:int[30];
	d:int;
	if (a < b){
		if (a == 5){
			d = a * (b - a)
		}
		c[d+1] = 1
	}
	return c
}

foo2(a: int): bool {
	while(a < 5){
		if (a == 3){
			a = 5 * 27
		}
		else {
			a = 21 % 2
	    }

	    a = a * 3
	}
	return true;
}

foo3(b: int[]) {
	if (b[4] != 5){
		while (b < 7){
			b = b + 1;
		}
	}
}
