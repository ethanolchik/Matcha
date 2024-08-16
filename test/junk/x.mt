module x;

struct Test {
    x: pub Float32
}

const PI: Float32 = 3.14;

func test(): Float32 {
    return PI;
}

export {
    Test,
    PI,
    test
}