// msvc
struct S {
    unsigned int i: 1;
    __declspec(align(128)) long j: 1;
};

static void f(void) {
    static_assert(sizeof(struct S) == 4, "");
    static_assert(_Alignof(struct S) == 4, "");
}
