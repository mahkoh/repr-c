// msvc
union X {
    long long b[];
};

union Y {
    long long : 0;
    char b[];
};

static void f(void) {
    static_assert(sizeof(union X) == 8, "");
    static_assert(sizeof(union Y) == 1, "");
}
