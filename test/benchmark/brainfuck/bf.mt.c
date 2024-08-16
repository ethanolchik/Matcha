#define MAINDEFINED
#include "matcha.h"
#include <stdio.h>
;
;
;
;
Int32 MATCHA__last(Int32* MATCHA__array) {
return MATCHA__array[(length_array(MATCHA__array) - 1)];
}

String MATCHA__interpretBrainfuck(String MATCHA__code, String MATCHA__input) {
Int32 MATCHA__memory[];
Int32 MATCHA__pointer = 0;
String MATCHA__output = "";
Int32 MATCHA__codeIndex = 0;
Int32 MATCHA__inputIndex = 0;
while ((MATCHA__codeIndex < strlen(MATCHA__code))) {
Char MATCHA__instruction = MATCHA__code[MATCHA__codeIndex];
if ((MATCHA__instruction == '+')) {
MATCHA__memory[MATCHA__pointer]+=1;
;
}
 else if ((MATCHA__instruction == '-')) {
MATCHA__memory[MATCHA__pointer]-=1;
;
}
 else if ((MATCHA__instruction == '>')) {
MATCHA__pointer+=1;
;
}
 else if ((MATCHA__instruction == '<')) {
MATCHA__pointer-=1;
;
}
 else if ((MATCHA__instruction == '.')) {
MATCHA__output=concat(MATCHA__output, chr(MATCHA__memory[MATCHA__pointer]));
;
}
 else if ((MATCHA__instruction == ',')) {
if ((MATCHA__inputIndex < strlen(MATCHA__input))) {
MATCHA__memory[MATCHA__pointer]=MATCHA__input[MATCHA__inputIndex];
;
MATCHA__inputIndex+=1;
;
}
 else {
MATCHA__memory[MATCHA__pointer]=0;
;
}
}
 else if ((MATCHA__instruction == '[')) {
if ((MATCHA__memory[MATCHA__pointer] == 0)) {
Int32 MATCHA__loopCount = 1;
while ((MATCHA__loopCount > 0)) {
MATCHA__codeIndex+=1;
;
if ((MATCHA__code[MATCHA__codeIndex] == '[')) {
MATCHA__loopCount+=1;
;
}
 else if ((MATCHA__code[MATCHA__codeIndex] == ']')) {
MATCHA__loopCount-=1;
;
}
}
}
}
 else if ((MATCHA__instruction == ']')) {
if ((MATCHA__memory[MATCHA__pointer] != 0)) {
Int32 MATCHA__loopCount = 1;
while ((MATCHA__loopCount > 0)) {
MATCHA__codeIndex-=1;
;
if ((MATCHA__code[MATCHA__codeIndex] == ']')) {
MATCHA__loopCount+=1;
;
}
 else if ((MATCHA__code[MATCHA__codeIndex] == '[')) {
MATCHA__loopCount-=1;
;
}
}
}
}
MATCHA__codeIndex+=1;
;
}
return MATCHA__output;
}

void MATCHA__benchmark(String MATCHA__code, String MATCHA__input) {
Float64 MATCHA__minTime = FLOAT64_MAX;
Float64 MATCHA__maxTime = FLOAT64_MIN;
const Int32 MATCHA__iterations = 100;
Float64 MATCHA__startTime = get_time();
Float64 MATCHA__totalExecutionTime = 0;
for (Int32 MATCHA__i = 0;
(MATCHA__i < MATCHA__iterations); MATCHA__i++) {
Float64 MATCHA__t1 = get_time();
String MATCHA__output = MATCHA__interpretBrainfuck(MATCHA__code, MATCHA__input);
Float64 MATCHA__t2 = get_time();
Float64 MATCHA__elapsed = (((MATCHA__t2 - MATCHA__t1)) / 100);
MATCHA__totalExecutionTime+=MATCHA__elapsed;
;
if ((MATCHA__elapsed < MATCHA__minTime)) {
MATCHA__minTime=MATCHA__elapsed;
;
}
if ((MATCHA__elapsed > MATCHA__maxTime)) {
MATCHA__maxTime=MATCHA__elapsed;
;
}
}
Float64 MATCHA__endTime = get_time();
Float64 MATCHA__totalTime = ((MATCHA__endTime - MATCHA__startTime));
Float64 MATCHA__averageTime = MATCHA__totalExecutionTime;
print("Benchmark results:\n");
print("Iterations: ");
println(int32ToString(MATCHA__iterations));
print("Total Time: ");
println(concat(float64ToString(MATCHA__totalTime), " seconds"));
print("Average Time: ");
println(concat(float64ToString(MATCHA__averageTime), " seconds"));
print("Min Time: ");
println(concat(float64ToString(MATCHA__minTime), " seconds"));
print("Max Time: ");
println(concat(float64ToString(MATCHA__maxTime), " seconds"));
}

Int32 MATCHA__main() {
String MATCHA__code = "++++++++[>+++++++++<-]+++++[>>++++++++++++++++++++>+++++++++++++++++++++>++++++++++++++++++++++>+++++++++++++++++++++++>++++++++++++++++++++++++<<<<<<-]++++[>>>>>>>++++++++<<<<<<<-]>.>+.->>--..+++.->>>.<<<<<<<+++[>+++++<-]>.>>>+.->-.+<--.++ <<.>>>>>+.-";
String MATCHA__input = "";
MATCHA__benchmark(MATCHA__code, MATCHA__input);
println(MATCHA__interpretBrainfuck(MATCHA__code, MATCHA__input));
return 0;
}


int main() {
    return matcha_init();
}
