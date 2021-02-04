// msvc
struct A {
    __declspec(align(128)) int i: 1;
};

struct B {
    struct A x;
};

#pragma pack(4)
struct C {
    struct A x;
};
#pragma pack()

#pragma pack(8)
struct D {
    struct A x;
};
#pragma pack()

#pragma pack(16)
struct E {
    struct A x;
};
#pragma pack()

#pragma pack(32)
struct F {
    struct A x;
};
#pragma pack()

static void f(void) {
#if defined(_M_IX86)
    static_assert(_Alignof(struct B) == 128, "");
    static_assert(_Alignof(struct C) == 4, "");
    static_assert(_Alignof(struct D) == 128, "");
    static_assert(_Alignof(struct E) == 128, "");
    static_assert(_Alignof(struct F) == 128, "");
#elif defined(_M_X64)
    static_assert(_Alignof(struct B) == 128, "");
    static_assert(_Alignof(struct C) == 4, "");
    static_assert(_Alignof(struct D) == 8, "");
    static_assert(_Alignof(struct E) == 128, "");
    static_assert(_Alignof(struct F) == 128, "");
#elif defined(_M_ARM)
    static_assert(_Alignof(struct B) == 8, "");
    static_assert(_Alignof(struct C) == 4, "");
    static_assert(_Alignof(struct D) == 8, "");
    static_assert(_Alignof(struct E) == 16, "");
    static_assert(_Alignof(struct F) == 8, "");
#elif defined(_M_ARM64)
    static_assert(_Alignof(struct B) == 8, "");
    static_assert(_Alignof(struct C) == 4, "");
    static_assert(_Alignof(struct D) == 8, "");
    static_assert(_Alignof(struct E) == 128, "");
    static_assert(_Alignof(struct F) == 8, "");
#else
    static_assert(0, "unknown target");
#endif
}
