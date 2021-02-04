// msvc
#include <stdlib.h>

struct S {
    char c: 1;
    int : 0;
    char d;
};

static void f(void) {
    static_assert(offsetof(
    struct S, d) == 4, "");
}
