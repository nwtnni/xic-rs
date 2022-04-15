use io

main(args: int[][]) {
  str:int[] = "ABC";
  println(str) // should be "ABC"
  str = "DEF"
  println(str) // should be "DEF"
  println("GHI") // should be "GHI"
}
