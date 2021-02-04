// msvc
__declspec(align(4)) typedef char Char;

struct A {
    Char a;
};

struct B {
    Char a: 1;
};

#pragma pack(1)

struct C {
    struct A a;
};

struct D {
    struct B a;
};

static void f(void) {
    static_assert(_Alignof(struct C) == 4, "");
    static_assert(_Alignof(struct D) == 1, "");
}
