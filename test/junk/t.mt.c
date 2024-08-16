#define MAINDEFINED
#include "matcha.h"
#include <stdio.h>
typedef struct MATCHA__Person {
String MATCHA__name;
} MATCHA__Person;
MATCHA__Person MATCHA__test() {
return {.MATCHA__name = "Ethan", };
}

Int32 MATCHA__main() {
MATCHA__Person MATCHA__p = {.MATCHA__name = "Ethan", };
println(concat("Hello, ", MATCHA__p.MATCHA__name));
return 0;
}


int main() {
    return matcha_init();
}
