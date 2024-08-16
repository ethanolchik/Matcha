#define MAINDEFINED
#include "matcha.h"
#include <stdio.h>
typedef struct MATCHA__x__MATCHA__Test {
Float32 MATCHA__x;
} MATCHA__x__MATCHA__Test;
const Float32 MATCHA__x__MATCHA__PI = 3.14;
Float32 MATCHA__x__MATCHA__test() {
return MATCHA__x__MATCHA__PI;
}

;
Float32 MATCHA__test() {
return MATCHA__x__MATCHA__PI;
}

void MATCHA__main() {
MATCHA__x__MATCHA__Test MATCHA__i = {.MATCHA__x = 3.14};
MATCHA__test();
}


int main() {
    return matcha_init();
}
