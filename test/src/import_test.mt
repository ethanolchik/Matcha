module import_test;

import x;

func test(): x.Test {
    return x.PI;
}

func main(): Void {
    // TODO: make sure type names exist
    var i: x.Test = x.Test;
}

export {
    main
}