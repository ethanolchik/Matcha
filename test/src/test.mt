module test;

struct Dog {
    name: String,
    colour: Colour
}

enum Colour {
    RED, GREEN, BLUE
}

func (Dog) new(name: String, colour: Colour): Dog {
    return Dog {
        name: name,
        colour: colour
    };
}

func (c: Colour) new(): Colour {
    return c;
}

func (d: Dog) bark(): pub String {
    return "Bark!";
}

func main(): Void {
    var myDog: Dog = Dog.new("Foo", Colour.RED);

    var myColour: Colour = Colour.new();

    myDog.bark();
}

export {
    Dog,
    Colour,

    main
}