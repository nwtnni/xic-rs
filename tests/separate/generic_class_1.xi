use vector

foo(): Vector::<int> {
    vector: Vector::<int> = new_vector::<int>()
    i: int = 0
    while i < 16 {
        vector.push(i)
        i = i + 1
    }
    return vector
}
