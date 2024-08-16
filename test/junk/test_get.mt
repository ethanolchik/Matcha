module test_get;

struct B {
    y: Int32
}

struct A {
    be: B
}

struct Point {
    a: A
}

struct Test {
    p: Point
}

func (p: Point) b(): A {
    return p.a;
}

func main(): Void {
    var t: Test = Test {
        p: Point {
            a: A {
                be: B {
                    y: 10
                }
            }
        }
    };

    t.p.b().be.y;
}

export {
    main
}