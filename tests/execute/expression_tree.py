import random
import numpy as np

binary = [
    "+",
    "*",
    "*>>",
    "/",
    "%",
    "+",
    "-"
]

class Tree:
    def __init__(self, left, value, right):
        self.left = left
        self.value = value
        self.right = right

    def xi(self):
        if self.left is None and self.right is None:
            return "{}".format(self.value)
        elif self.right is None:
            return "{}{}".format(self.left, self.value.xi())
        elif self.left == "(" and self.right == ")":
            return "({})".format(self.value.xi())
        else:
            return "({} {} {})".format(self.left.xi(), self.value, self.right.xi())

    def python(self):
        if self.left is None and self.right is None:
            return "np.int64({})".format(self.value)
        elif self.right is None:
            return "{}{}".format(self.left, self.value.python())
        elif self.left == "(" and self.right == ")":
            return "({})".format(self.value.python())
        elif self.value == "*>>":
            return "np.int64(({}.item() * {}.item()) >> 64)".format(self.left.python(), self.right.python())
        elif self.value == "/":
            return "np.int64({} / {})".format(self.left.python(), self.right.python())
        elif self.value == "%":
            return "np.fmod({}, {})".format(self.left.python(), self.right.python())
        else:
            return "({} {} {})".format(self.left.python(), self.value, self.right.python())

def generate(depth, large):
    switch = random.random()

    if depth == 0 or switch < 0.2:
        if depth == 0 or not large:
            return Tree(None, random.randint(-1000, 1000), None)
        else:
            return Tree(None, random.randint(-9223372036854775808, 9223372036854775807), None)
    elif switch < 0.3:
        return Tree("-", generate(depth - 1, large), None)
    elif switch < 0.9:
        return Tree(generate(depth - 1, large), random.choice(binary), generate(depth - 1, large))
    else:
        return Tree("(", generate(depth - 1, large), ")")

if __name__ == "__main__":
    expression = generate(10, True)
    print("    print(\"{} = \")".format(eval(expression.python())))
    print("    println(unparseInt({}))".format(expression.xi()))
