// msvc
struct S {
    _Bool v: 8;
};

static void f(void) {
    static_assert(sizeof(struct S) == 1, "");
}
