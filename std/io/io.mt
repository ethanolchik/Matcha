module io;

/// Prints a string to the console with a newline.
func println(str: String): extern Void;

/// Prints a string to the console.
func print(str: String): extern Void;


export {
    println,
    print
}