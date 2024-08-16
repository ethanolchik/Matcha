module test_get_call;


struct Test {
    x: Int32
}

func (t: Test) a(): Void {
    t.x + 1;
}

func main(): Int32 {
    var t: Test = Test {
        x: 1
    };

    t.a();
    
    return 0;
}