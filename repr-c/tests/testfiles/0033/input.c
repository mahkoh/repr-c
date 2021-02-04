// msvc
#include <stdlib.h>

struct A {
    char c;
    long a;
};

static void f(void) {
    static_assert(offsetof(struct A, a) == 4, "");
}
