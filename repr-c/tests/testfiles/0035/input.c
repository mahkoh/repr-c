// msvc
union X {
    int : 0;
    char : 1;
};

union Y {
    char : 1;
    int : 0;
};

static void f(void) {
    static_assert(sizeof(union X) == 1, "");
    static_assert(sizeof(union Y) == 4, "");
}
