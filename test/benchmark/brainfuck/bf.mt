module brainfuck;

import std.io;

func strlen(str: String): extern Int32;
func push_array(array: Int32[], value: Int32): extern Void;
func length_array(array: Int32[]): extern Int32;
func pop_array(array: Int32[]): extern Int32;
func insert_array(array: Int32[], index: Int32, value: Int32): extern Void;
func chr(n: Int32): extern String;
func concat(str1: String, str2: String): extern String;
func int32ToString(n: Int32): extern String;
func float64ToString(n: Float64): extern String;

const FLOAT64_MAX: extern Float64;
const FLOAT64_MIN: extern Float64;

func get_time(): extern Float64;

func last(array: Int32[]): Int32 {
    return array[length_array(array) - 1];
}

func interpretBrainfuck(code: String, input: String): String {
    var memory: Int32[];
    var pointer: Int32 = 0;
    var output: String = "";

    var codeIndex: Int32 = 0;
    var inputIndex: Int32 = 0;

    while (codeIndex < strlen(code)) {
        var instruction: Char = code[codeIndex];

        if (instruction == '+') {
            memory[pointer] += 1;
        } else if (instruction == '-') {
            memory[pointer] -= 1;
        } else if (instruction == '>') {
            pointer += 1;
        } else if (instruction == '<') {
            pointer -= 1;
        } else if (instruction == '.') {
            output = concat(output, chr(memory[pointer]));
        } else if (instruction == ',') {
            if (inputIndex < strlen(input)) {
                memory[pointer] = input[inputIndex];
                inputIndex += 1;
            } else {
                memory[pointer] = 0;
            }
        } else if (instruction == '[') {
            if (memory[pointer] == 0) {
                var loopCount: Int32 = 1;
                while (loopCount > 0) {
                    codeIndex += 1;
                    if (code[codeIndex] == '[') {
                        loopCount += 1;
                    } else if (code[codeIndex] == ']') {
                        loopCount -= 1;
                    }
                }
            }
        } else if (instruction == ']') {
            if (memory[pointer] != 0) {
                var loopCount: Int32 = 1;
                while (loopCount > 0) {
                    codeIndex -= 1;
                    if (code[codeIndex] == ']') {
                        loopCount += 1;
                    } else if (code[codeIndex] == '[') {
                        loopCount -= 1;
                    }
                }
            }
        }

        codeIndex += 1;
    }

    return output;
}

func benchmark(code: String, input: String): Void {
    var minTime: Float64 = FLOAT64_MAX;
    var maxTime: Float64 = FLOAT64_MIN;
    const iterations: Int32 = 100;

    var startTime: Float64 = get_time();
    var totalExecutionTime: Float64 = 0;

    for (var i: Int32 = 0; i < iterations; i++) {
        var t1: Float64 = get_time();
        var output: String = interpretBrainfuck(code, input);
        var t2: Float64 = get_time();

        var elapsed: Float64 = (t2 - t1) / 100;

        totalExecutionTime += elapsed;

        if (elapsed < minTime) {
            minTime = elapsed;
        }

        if (elapsed > maxTime) {
            maxTime = elapsed;
        }
    }

    var endTime: Float64 = get_time();
    var totalTime: Float64 = (endTime - startTime);

    var averageTime: Float64 = totalExecutionTime;

    io.print("Benchmark results:\n");
    io.print("Iterations: "); io.println(int32ToString(iterations));
    io.print("Total Time: "); io.println(concat(float64ToString(totalTime), " seconds"));
    io.print("Average Time: "); io.println(concat(float64ToString(averageTime), " seconds"));
    io.print("Min Time: "); io.println(concat(float64ToString(minTime), " seconds"));
    io.print("Max Time: "); io.println(concat(float64ToString(maxTime), " seconds"));
}

func main(): Int32 {
    var code: String = "++++++++[>+++++++++<-]+++++[>>++++++++++++++++++++>+++++++++++++++++++++>++++++++++++++++++++++>+++++++++++++++++++++++>++++++++++++++++++++++++<<<<<<-]++++[>>>>>>>++++++++<<<<<<<-]>.>+.->>--..+++.->>>.<<<<<<<+++[>+++++<-]>.>>>+.->-.+<--.++ <<.>>>>>+.-";
    var input: String = "";

    benchmark(code, input);

    io.println(interpretBrainfuck(code, input));

    return 0;
}