// msvc
#pragma pack(1)

__declspec(align(8)) enum E {
        A = 1,
        B = 0xffff0fffffff,
};

static void f(void) {
        static_assert(sizeof(enum E) == 4, "");
        static_assert(_Alignof(enum E) == 8, "");
        static_assert(B == 0x0fffffff, "");
}
