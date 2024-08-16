module fizzbuzz;

func println(str: String): extern Void;
func int32ToString(n: Int32): extern String;


func fizzbuzz(n: Int32): String {
    if (n % 15 == 0) {
        return "FizzBuzz";
    } else if (n % 3 == 0) {
        return "Fizz";
    } else if (n % 5 == 0) {
        return "Buzz";
    } else {
        return int32ToString(n);
    }
}

func main(): Int32 {
    for (var i: Int32 = 1; i <= 100; i++) {
        println(fizzbuzz(i));
    }

    return 0;
}