#define MAINDEFINED
#include "matcha.h";
#include <stdio.h>;
typedef struct MATCHA__Test {
Int32 MATCHA__x;
} MATCHA__Test;
void MATCHA__a(MATCHA__Test *MATCHA__t) {
(MATCHA__t->MATCHA__x + 1);
}

Int32 MATCHA__main() {
MATCHA__Test MATCHA__t = {.MATCHA__x = 1, };
MATCHA__a(&MATCHA__t);
return 0;
}


int main() {
    return matcha_init();
}
