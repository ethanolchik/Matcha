module hello_world;

func println(str: String): extern Void;

func main(): Int32 {
    println("Hello, World!");
    return 0;
}