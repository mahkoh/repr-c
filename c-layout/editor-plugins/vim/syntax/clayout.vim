if exists("b:current_syntax")
    finish
endif

syn keyword clayoutKeyword const typedef _ BITS_PER_BYTE
syn keyword clayoutAnnotation pragma_pack attr_packed align
syn keyword clayoutFunction sizeof alignof offsetof offsetof_bits
syn keyword clayoutType opaque enum struct union () bool u8 u16 u32 u64 u128 \
                        i8 i16 i32 i64 i128 char signed unsigned short int \
                        long f32 f64 float double ptr
syn match clayoutDelimiter "(\|)\|,\|\[\|\]\|\.\|{\|}"
syn match clayoutOperator "=\|==\|!=\|<=\|>=\|<\|>\|+\|-\|*\|/\|%\|!\|-\|||\|&&"
syn match clayoutIdentifier "\[a-zA-Z_\]\+\[a-zA-Z_0-9\]\*"
syn match clayoutDigit "\d"
syn match clayoutLineComment "//.\*$"

hi def link clayoutKeyword     Keyword
hi def link clayoutAnnotation  Keyword
hi def link clayoutFunction    Function
hi def link clayoutType        Keyword
hi def link clayoutDelimiter   Delimiter
hi def link clayoutOperator    Operator
hi def link clayoutIdentifier  Identifier
hi def link clayoutDigit       Number
hi def link clayoutLIneComment Comment

let b:current_syntax = "clayout"
