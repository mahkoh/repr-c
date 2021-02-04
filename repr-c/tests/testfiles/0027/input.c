// msvc
#include <stdlib.h>

struct Y {
    char c;
    int i;
};

static void f(void) {
    static_assert(offsetof(struct Y, i) == 4, "");
}
