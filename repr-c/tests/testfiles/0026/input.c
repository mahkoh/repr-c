// msvc
struct X {
    __declspec(align(2)) long long b[];
};

struct Y {
    __declspec(align(8)) long long b[];
};

static void f(void) {
    static_assert(sizeof(struct X) == 4, "");
    static_assert(_Alignof(struct X) == 8, "");

    static_assert(sizeof(struct Y) == 8, "");
}
