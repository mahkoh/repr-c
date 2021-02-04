// msvc

__declspec(align(2)) typedef int A;
__declspec(align(8)) typedef int B;

#pragma pack(1)

struct X {
    A a;
};

struct Y {
    B b;
};

static void f(void) {
    static_assert(sizeof(A) == 4, "");
    static_assert(_Alignof(A) == 4, "");

    static_assert(sizeof(struct X) == 4, "");
    static_assert(_Alignof(struct X) == 2, "");

    static_assert(sizeof(B) == 4, "");
    static_assert(_Alignof(B) == 8, "");

    static_assert(sizeof(struct Y) == 8, "");
    static_assert(_Alignof(struct Y) == 8, "");
}
