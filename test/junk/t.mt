module t;

func println(str: String): extern Void;
func concat(a: String, b: String): extern String;

struct Person {
    name: String
}

func main(): Int32 {
    var p: Person = Person {
        name: "Ethan"
    };

    println(concat("Hello, ", p.name));

    return 0;
}