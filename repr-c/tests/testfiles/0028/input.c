// msvc
__declspec(align(4)) typedef char Char;

#pragma pack(1)

struct A {
    Char a;
};

struct B {
    __declspec(align(4)) char a;
};

struct C {
    __declspec(align(8)) Char a;
};

struct D {
    __declspec(align(2)) Char a;
};

static void f(void) {
    static_assert(_Alignof(struct A) == 4, "");
    static_assert(_Alignof(struct B) == 4, "");
    static_assert(_Alignof(struct C) == 8, "");
    static_assert(_Alignof(struct D) == 4, "");
}
