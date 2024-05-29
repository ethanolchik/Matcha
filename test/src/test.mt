module test;

struct Dog {
    name: String,
    colour: Colour
}

enum Colour {
    RED, GREEN, BLUE
}

func new_dog(name: String, colour: Colour): Dog {
    return Dog {
        name: name,
        colour: colour
    };
}

func (d: Dog) bark(): pub String {
    return "Bark!";
}

func main(): Void {
    var myDog: Dog = new_dog("Foo", Colour.RED);

    myDog.bark();
}

export {
    Dog,
    Colour,
    
    new_dog,

    main
}