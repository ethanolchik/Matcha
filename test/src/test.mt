func main(): Void {
    var z: Test = Test { x: 3 };
    z.x;
}

struct A {
    x: Float32
}

struct Test {
    x: Int32
}

const AA: A = A { x: 3.14 };