// msvc
#pragma pack(1)

struct S {
    char c;
    int : 1;
};

static void f(void) {
    static_assert(sizeof(struct S) == 5, "");
}
