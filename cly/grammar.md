The input is first transformed into tokens. The tokens are the terminals in the grammar
shown below. Before a token is produced, all whitespace, including comments, is removed
from the start of the remaining input. If multiple terminals match the start of the
remaining input, the longest terminal is used.

The thus produced list of tokens is parsed according to the following grammar.

```peg
Top <- Declaration*

Declaration <- ConstDeclaration / TypeDeclaration
ConstDeclaration <- 'const' Identifier '=' Expression
TypeDeclaration <- Identifier '=' Type

Keyword <- 'const' / 'typedef' / 'BITS_PER_BYTE' / 'pragma_pack' / 'attr_packed' / 'align'
         / 'sizeof' / 'sizeof_bits' / 'offsetof' / 'offsetof_bits' / 'opaque' / 'enum'
         / 'struct' / 'union' / 'unit' / 'bool' / 'u8' / 'i8' / 'u16' / 'i16' / 'u32'
         / 'i32' / 'u64' / 'i64' / 'u128' / 'i128' / 'char' / 'signed' / 'unsigned'
         / 'short' / 'int' / 'long' / 'f32' / 'f64' / 'float' / 'double' / 'ptr'

Identifier <- !Keyword ([a-zA-Z] [a-zA-Z_0-9]* / '_' [a-zA-Z_0-9]+)

Number <- BinaryNumber / OctalNumber / DecimalNumber / HexadecimalNumber
BinaryNumber <- '0b' [01_]* [01] [01_]*
OctalNumber <- '0o' [0-7_]* [0-7] [0-7_]*
DecimalNumber <- [0-9_]* [0-9] [0-9_]*
HexadecimalNumber <- '0x' [0-9a-fA-F_]* [0-9a-fA-F] [0-9a-fA-F_]*

Expression <- AtomicExpression (BinaryOperator AtomicExpression)*
AtomicExpression <- '-' AtomicExpression
                  / '!' AtomicExpression
                  / '(' Expression ')'
                  / 'BITS_PER_BYTE'
                  / Number
                  / ('sizeof' / 'sizeof_bits') '(' Type ')'
                  / Identifier
                  / ('offsetof' / 'offsetof_bits') '(' Type ',' OffsetofPath ')'
OffsetofPath <- (Identifier / '[' Expression ']')
                    ('.' Identifier / '[' Expression ']')*
BinaryOperator <- '==' / '!=' / '<=' / '>=' / '||' / '&&' / '>' / '<' / '+' / '-' / '*'
                / '/' | '%'
SimpleExpression <- '-'? Number
                
Type <- TypeLayout<SimpleExpression>? Annotation* TypeVariant
TypeVariant <- Identifier
             / Typedef
             / OpaqueType
             / Enum
             / Struct
             / Union
             / Array
             / BuiltinType
Typedef <- 'typedef' Type
OpaqueType <- 'opaque' TypeLayout<Expression>
Enum <- 'enum' '{' (Expression ',')* Expression? '}'
Struct <- 'struct' RecordBody
Union <- 'union' RecordBody
Array <- '[' Expression? ']' Type
RecordBody <- '{' (RecordField ',')* RecordField? '}'
RecordField <- FieldLayout? Annotation* ('_' / Identifier) Type
BuiltinType <- 'unsigned' 'long' 'long' / 'signed' 'long' 'long' / 'long' 'long'
             / 'signed' 'char' / 'signed' 'short' / 'signed' 'int' / 'signed' 'long'
             / 'unsigned' 'char' / 'unsigned' 'short' / 'unsigned' 'int'
             / 'unsigned' 'long' / 'unit' / 'bool' / 'u8' / 'i8' / 'u16' / 'i16' / 'u32'
             / 'i32' / 'u64' / 'i64' / 'u128' / 'i128' / 'char' / 'signed' / 'unsigned'
             / 'short' / 'int' / 'long' / 'f32' / 'f64' / 'float' / 'double' / 'ptr'
TypeLayout<T> <- '{' (TypeLayoutElement<T> ',')* TypeLayoutElement<T>? '}'
TypeLayoutElement<T> <- (  'size'
                        / 'alignment'
                        / 'field_alignment'
                        / 'pointer_alignment'
                        / 'required_alignment'
                        )
                        ':' T
FieldLayout <- '{' (FieldLayoutElement ',')* FieldLayoutElement? '}'
FieldLayoutElement <- ('size' / 'offset') ':' SimpleExpression
Annotation <- '@' ( 'attr_packed'
                  / ('align' ('(' Expression ')')?)
                  / ('pragma_pack' '(' Expression ')')
                  )
```
