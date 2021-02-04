// msvc
union U {
    int a: 1;
};

static void f(void) {
    static_assert(_Alignof(union U) == 1, "");
}
