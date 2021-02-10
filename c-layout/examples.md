## Basic Structure

```c
// Comments start with two '/' and go to the end of the line

// Types are declared by writing the name of the declaration followed by the type:
MyType = int

// Constants are declared by writing the keyword 'const' followed by the name of the
// declaration followed by an expression:
const MyConstant = 1 + 1
```

## Simple Types

```c
// All of the usual builtin C type are available
Bool = bool  // _Bool in C
Char = char
SignedChar = signed char
UnsignedChar = unsigned char
Short = short
UnsignedShort = unsigned short
Int = int
UnsignedInt = unsigned int
Long = long
UnsignedLong = unsigned long
LongLong = long
UnsignedLongLong = unsigned long long
Float = float
Double = double
Ptr = ptr // all kinds of pointers

// Additionally, all of the builtin Rust types are available
Unit = unit // () in Rust
U8 = u8
I8 = i8
U16 = u16
I16 = i16
U32 = u32
I32 = i32
U64 = u64
I64 = i64
U128 = u128
I128 = i128
F32 = f32
F64 = f64
```

## Typedefs

```c
// Typedefs are written with the typedef keyword
MyTypedefInt = typedef int
        
// The difference between MyTypedefInt and
MyPlainInt = int
// is that MyTypedefInt can have annotations:        
MyTypedefIntWithAnnotations = @align(8) typedef int
// We'll learn more about annotations below
```

## Structs and Unions

```c
// Structs and unions are written with the struct or union keyword
MyStruct = struct {
    i int,
    c char,
}
MyUnion = union {
    i int,
    c char,
}

// Structs and unions contains zero or more fields. Each field can be prefixed with annotations:
MyStructWithFieldAnnotations = struct {
    @attr_packed i int,
    c char,
}
// We'll learn more about annotations below.

// Each field in a struct or union can be a bit-field. A bit-field is written like this:
MyStructWithABitField = struct {
    c char,
    i int:4,
}
// The number after the colon can be an arbitrary expression:
MyStructWithABitField2 = struct {
    c char,
    i int:2+2,
}

// The identifier in a bit-field can be replaced by _:
MyStructWithABitField3 = struct {
    c char,
    _ int:4
}
// This corresponds to an unnamed bit-field in C.
// A bit-field with width 0 must be unnamed.
```

## Arrays

```c
// Arrays can take one of two forms. With a size:
MyArray = [1]int
// Or without a size:
MyArrayWithoutASize = []int
```

## Nested Types

```c
// Types can be arbitrarily nested:
MyType = int
MyStruct = struct {
    my_type MyType,
    my_undeclared_type [3]union {
        i int,
        j MyType:4,
    }
}
```

## Enums

```c
MyEnum = enum {
    1,
    2,
    7,
    sizeof(int),
    9,
}
```

## Annotations

```c
// There are three annotations:

// @pragma_pack corresponds to #pragma pack in C
MyPackedStruct = @pragma_pack(4) struct {
    l long,
}

// @attr_packed corresponds to __attribute__((packed)) in C
MyPackedStruct2 = @attr_packed struct {
    l long,
}

// @align(N) corresponds to __attribute__((aligned(N))) or __declspec(align(N)) in C
MyAlignedStruct = @align(8) struct {
    s short,
}
// @align can also be used without (N). In this case it corresponds to
// __attribute__((aligned)) in C.
MyAlignedStruct2 = @align struct {
    s short,
}

// These annotations can be used on typedefs, structs, unions, and enums. They cannot be used
// on simple types (e.g. `int`), arrays, opaque types, or plain type references.
// ```
// DoesNotWork = struct {
//     a @align(1) int,
//     b @align(1) [1]int,
//     c @align(1) MyPackedStruct,
//     d @align(1) opaque { size: 1, alignment: 1 },
// }
// ```
// Opaque types are explain further below.

// Furthermore, the @align and @attr_packed annotations can be used on fields:
MyStructWithFieldAnnotations = struct {
    @align(8) i int,
    @attr_packed j long,
}

// All annotations except for @pragma_pack can occur multiple times on the same type or
// field. @pragma_pack can occur at most once.
```
