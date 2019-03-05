bar(): int[] {
  return {1, 2}  
}

bar2(): int[][] {
  return {{}, {}}  
}

bar3(): bool[][][] {
  return {{}, {{true, false}}}  
}

foo() {	
  _ = bar();
  _ = bar2();
  _ = bar3();
}
