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

#define FLOAT64_MAX 1.7976931348623157e+308
#define FLOAT64_MIN 2.2250738585072014e-308

Float64 get_time()
{
    struct timeval t;
    struct timezone tzp;
    gettimeofday(&t, &tzp);
    return t.tv_sec + t.tv_usec*1e-6;
}

Int32 matcha_init() {
    #ifdef MAINDEFINED
        return MATCHA__main();
    #endif
    return 0;
}

Void println(String str) {
    printf("%s\n", str);
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

Void push_array(Int32 *array, Int32 value) {
    array = (Int32 *)realloc(array, sizeof(Int32) * (value + 1));
}

Int32 length_array(Int32 *array) {
    return sizeof(array) / sizeof(array[0]);
}

Int32 pop_array(Int32 *array) {
    Int32 value = array[length_array(array) - 1];
    array = (Int32 *)realloc(array, sizeof(Int32) * (length_array(array) - 1));
    return value;
}

Void insert_array(Int32 *array, Int32 index, Int32 value) {
    Int32 *new_array = (Int32 *)malloc(sizeof(Int32) * (length_array(array) + 1));
    for (Int32 i = 0; i < length_array(array); i++) {
        if (i < index) {
            new_array[i] = array[i];
        } else {
            new_array[i + 1] = array[i];
        }
    }
    new_array[index] = value;
    array = new_array;
}

String chr(Int32 n) {
    char *str = (char*)malloc(2);
    str[0] = n;
    str[1] = '\0';
    return str;
}

String concat(String str1, String str2) {
    String result = (String)malloc(strlen(str1) + strlen(str2) + 1); // +1 for the null-terminator
    strcpy(result, str1);
    strcat(result, str2);
    return result;
}

Void print(String str) {
    printf("%s", str);
}

#endif