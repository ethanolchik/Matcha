module array;

func int32ToString(n: Int32): extern String;
func println(str: String): extern Void;

func main(): Int32 {
    var x: Int32[] = [1, 2, 3, 4, 5];

    println(int32ToString(x[0]));
    return 0;
}