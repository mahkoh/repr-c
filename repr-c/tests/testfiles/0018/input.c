// msvc

__declspec(align(4)) typedef char Char;

typedef Char X[3];

static void f(void) {
    static_assert(sizeof(X) == 3, "");
    static_assert(_Alignof(X) == 4, "");
}
