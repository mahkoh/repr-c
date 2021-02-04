// msvc
#pragma pack(1)

struct S {
    __declspec(align(4)) char c: 1;
};

typedef struct S X[3];

static void f(void) {
    static_assert(sizeof(X) == 3, "");
    static_assert(_Alignof(X) == 4, "");
}
