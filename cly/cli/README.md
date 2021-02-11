# cly

cly is a program that calculates the layout of C types using a domain-specific language.

# Example

```
~$ cat input

const C = sizeof(char) + sizeof(long)
A = @pragma_pack(2) struct {
    c char,
    @align(4) i int:3,
    j int:5,
    k [C]short,
}

~$ cly --target x86_64-unknown-linux-gnu input

const C = {9}sizeof(char) + sizeof(long)
A = { size: 176, alignment: 16 }@pragma_pack(2) struct {
    { offset: 0, size: 8 }c { size: 8, alignment: 8 }char,
    { offset: 16, size: 3 }@align(4) i { size: 32, alignment: 32 }int:3,
    { offset: 19, size: 5 }j { size: 32, alignment: 32 }int:5,
    { offset: 32, size: 144 }k { size: 144, alignment: 16 }[{9}C]{ size: 16, alignment: 16 }short,
}

~$ cly --target i686-pc-windows-msvc input

const C = {5}sizeof(char) + sizeof(long)
A = { size: 144, field_alignment: 32, pointer_alignment: 16 }@pragma_pack(2) struct {
    { offset: 0, size: 8 }c { size: 8, alignment: 8 }char,
    { offset: 32, size: 3 }@align(4) i { size: 32, alignment: 32 }int:3,
    { offset: 35, size: 5 }j { size: 32, alignment: 32 }int:5,
    { offset: 64, size: 80 }k { size: 80, alignment: 16 }[{5}C]{ size: 16, alignment: 16 }short,
}
```

See [examples.md](../examples.md) for a full description of the program input and output.

# Installation

You can install cly with [cargo](https://rustup.rs):

```
~$ cargo install cly
```

# Grammar

The full grammar of the domain-specific language is described in [grammar.md](../grammar.md)

# License

cly is licensed under the GPLv3 or later.
