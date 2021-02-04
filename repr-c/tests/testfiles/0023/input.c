// msvc
__declspec(align(8)) struct X {
    char c;
};

#pragma pack(1)

struct Y {
    struct X x;
};

static void f(void) {
    static_assert(_Alignof(struct Y) == 8, "");
}
