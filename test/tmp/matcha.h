#ifndef MATCHA_H
#define MATCHA_H

#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/time.h>
#include <sys/resource.h>

typedef int Int32;
typedef long Int64;
typedef float Float32;
typedef double Float64;
typedef char * String;
typedef char Char;
typedef void Void;
typedef bool Bool;

#define FLOAT32_MIN 1.175494351e-38
#define FLOAT32_MAX 3.402823466e+38

#define FLOAT64_MAX 1.7976931348623157e+308
#define FLOAT64_MIN 2.2250738585072014e-308

Int32 matcha_init() {
    #ifdef MAINDEFINED
        return MATCHA__main();
    #endif
    return 0;
}

Void println(String str) {
    printf("%s\n", str);
}

Void print(String str) {
    printf("%s", str);
}

String slice(String str, Int32 start, Int32 end) {
    String result = (String)malloc(end - start + 1);
    strncpy(result, str + start, end - start);
    result[end - start] = '\0';
    return result;
}

String concat(String str1, String str2) {
    String result = (String)malloc(strlen(str1) + strlen(str2) + 1); // +1 for the null-terminator
    strcpy(result, str1);
    strcat(result, str2);
    return result;
}

String int32ToChar(Int32 n) {
    char *str = (char*)malloc(2);
    str[0] = n;
    str[1] = '\0';
    return str;
}

String int64ToChar(Int64 n) {
    char *str = (char*)malloc(2);
    str[0] = n;
    str[1] = '\0';
    return str;
}

String int32ToStr(Int32 n) {
    char *str = (char*)malloc(12);
    sprintf(str, "%d", n);
    return str;
}

String int64ToStr(Int64 n) {
    char *str = (char*)malloc(12);
    sprintf(str, "%ld", n);
    return str;
}

String float32ToStr(Float32 n) {
    char *str = (char*)malloc(12);
    sprintf(str, "%.12f", n);
    return str;
}

String float64ToStr(Float64 n) {
    char *str = (char*)malloc(12);
    sprintf(str, "%.12f", n);
    return str;
}

String boolToStr(Bool b) {
    return b ? "true" : "false";
}

Int32 boolToInt32(Bool b) {
    return b ? 1 : 0;
}

Int32 strToInt32(String str) {
    return atoi(str);
}

Int64 strToInt64(String str) {
    return atol(str);
}

Float32 strToFloat32(String str) {
    return atof(str);
}

Float64 strToFloat64(String str) {
    return atof(str);
}

String charToString(Char c) {
    char *str = (char*)malloc(2);
    str[0] = c;
    str[1] = '\0';
    return str;
}

#endif