// msvc
union X {
    long long : 0;
};

union Y {
    char : 0;
};

static void f(void) {
    static_assert(sizeof(union X) == 4, "");
    static_assert(sizeof(union Y) == 4, "");
}
