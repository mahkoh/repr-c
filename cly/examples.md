This document contains two sections:

- Input Examples explains every feature of the language.
- Output Examples explains the program output.

# Input Examples

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

## Opaque Types

```c
// Opaque types allow you to declare a type with an almost arbitrary layout:
MyOpaqueType = opaque { size: 128, alignment: 8 }
// NOTE that all values inside declaration of the opaque type is in BIT units.
// The above declaration declares an opaque type with size 8 bytes and alignment 1 byte.

// Inside the `{` of an opaque type, the following keys can occur:
// - size: defines the size of the type,
// - alignment: defines the alignment of the type,
// - field_alignment: defines the field alignment of the type,
// - pointer_alignment: defines the pointer alignment of the type,
// - required_alignment: defines the required alignment of the type
//
// `size` must always be defined.
// `required_alignment` is optional. If it is not defined it defaults to 8 bits.
// `alignment` is a shortcut for setting both `field_alignment` and `pointer_alignment`
//      to the same value. If `alignment` is defined, those two fields must not be defined.
// If `alignment` is not defined, then both `field_alignment` and `pointer_alignment`
//      must be defined.
MyExtensiveOpaqueType = opaque {
    size: 48,
    field_alignment: 32,
    pointer_alignment: 16,
    required_alignment: 16,
}
```

## Expressions

```c
// All of the usual expressions are supported:
const A = 1 + 2 * 3 - 4 / 5 % 6 + -7 + !!9
const B = A > 0 && A < 100 || A == 33
// Note that the boolean expressions treat 0 as false and all other numbers as true.
// They evaluate themselves to 0 or 1.

// Number literals can be in binary, octal, decimal, or hexadecimal form
const C = 0b1010_1010 // _ is supported as a separator within numbers
const D = 0o077
const E = 1237
const F = 0xff

// Because we sometimes have to translate between bit units and byte units,
// the builtin constant BITS_PER_BYTE evaluates to 8
const G = BITS_PER_BYTE == 8
        
// The builtin functions sizeof and sizeof_bits can be used to get the size of a type
const H = sizeof(int)
const I = sizeof_bits(long long)
        
// The builtin functions offsetof and offsetof_bits can be used to get the offset of a field
J = struct {
    c char,
    i int,
    j [2]struct {
        a int:1,
        b int:1,
    }
}
const K = offsetof(J, i)
const L = offsetof_bits(J, j[1].b)
// Note that, since a bit-field might not start at a multiple of 8 bits, offsetof cannot
// be applied to bit-fields.
```

# Output Examples

The output of the program is its input but with annotations that describe the layout of
types and fields and the values of expressions. All of the following outputs are for the
`x86_64-pc-windows-msvc` target.

These annotations are written by prefixing the types/fields/expressions with their
properties enclosed in curly braces:

```c
MyType = { size: 32, alignment: 32 }int
// The value of a constant is written in curly braces in front of the expression.
const MyConstant = {2}1 + 1
```

The properties of types can take various forms:

```c
// In the simplest case, the properties contain the size and alignment of the type, in bits.
MyPlainInt = { size: 32, alignment: 32 }int

// On MSVC targets only, a type can have a required alignment. This is the maximum of the
// alignments requested with @align on the type itself or one of its contained types.
// The required alignment is only printed if it is not 8 bits. On non-MSVC targets, it is
// always 8 bits.
MyAlignedTypedef = { size: 32, alignment: 32, required_alignment: 32 }@align(4) typedef { size: 32, alignment: 32 }int
        
// @align annotations can cause the alignment of a type to become larger than its size.
// In this case, if the type is contained in an array, the second element of the array
// will not have the alignment requested by the @align annotation. This means that pointers
// in C are not always guaranteed to have the alignment requested by @align annotations.
// We therefore distinguish between the alignment of a type when it is used as a field
// and the alignment of pointers to the type.
//
// When those two values are the same, they are printed with the name `alignment`. If they
// are not the same, we print them as `field_alignment` and `pointer_alignment`.
//
// `field_alignment` almost always corresponds to the value returned by `_Alignof` in C.
// `pointer_alignment` corresponds to the value returned by `std::mem::align_of` in Rust.
MySuperAlignedTypedef = { size: 32, field_alignment: 64, pointer_alignment: 32, required_alignment: 64 }@align(8) typedef { size: 32, alignment: 32 }int
```

Fields describe their position and size in bits:

```c
J = { size: 128, alignment: 32 }struct {
    { offset: 0, size: 8 }c { size: 8, alignment: 8 }char,
    { offset: 32, size: 32 }i { size: 32, alignment: 32 }int,
    { offset: 64, size: 64 }j { size: 64, alignment: 32 }[2]{ size: 32, alignment: 32 }struct {
        { offset: 0, size: 1 }a { size: 32, alignment: 32 }int:1,
        { offset: 1, size: 1 }b { size: 32, alignment: 32 }int:1,
    }
}
const L = {97}offsetof_bits(J, j[1].b)
```

