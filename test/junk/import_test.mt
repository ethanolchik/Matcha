module import_test;

import x;

func test(): Float32 {
    return x.PI;
}

func main(): Void {
    var i: x.Test = x.Test {
        x: 3.14
    };

    x.test();
}

export {
    main
}