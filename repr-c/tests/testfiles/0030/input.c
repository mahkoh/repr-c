// msvc
#pragma pack(2)

struct A {
    int a;
};

static void f(void) {
    static_assert(_Alignof(struct A) == 2, "");
}
