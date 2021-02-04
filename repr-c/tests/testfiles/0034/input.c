// msvc
union U {
    int l;
    char c;
};

static void f(void) {
    static_assert(sizeof(union U) == 4, "");
}
