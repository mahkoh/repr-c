// msvc
struct A {
    long a;
    char c;
};

static void f(void) {
    static_assert(_Alignof(struct A) == 4, "");
    static_assert(sizeof(struct A) == 8, "");
}
