// msvc
__declspec(align(4)) typedef char Char;

#pragma pack(2)

struct A {
    Char a;
};

static void f(void) {
    static_assert(_Alignof(struct A) == 4, "");
}
