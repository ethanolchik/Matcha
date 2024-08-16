#ifndef MATCHA_H
#define MATCHA_H

#include <stdbool.h>
#include <stdio.h>

typedef int Int32;
typedef long Int64;
typedef float Float32;
typedef double Float64;
typedef char * String;
typedef void Void;
typedef bool Bool;


Int32 matcha_init() {
    #ifdef MAINDEFINED
        return MATCHA__main();
    #endif
    return 0;
}

Void println(String str) {
    printf("%s\n", str);
}

String int32ToString(Int32 n) {
    char *str = malloc(12);
    sprintf(str, "%d", n);
    return str;
}

Void pushArray(Int32 *arr, Int32 n) {
    arr = realloc(arr, sizeof(Int32) * (n + 1));
}

#endif