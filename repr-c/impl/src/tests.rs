use crate::layout::{Type, Annotation, TypeVariant, BuiltinType, TypeLayout, Array, Record, RecordKind, RecordField};
use crate::builder::compute_layout;
use crate::target::Target;
use crate::result::{ErrorType};

#[test]
fn annotated_builtin() {
    let ty = Type::<()> {
        layout: (),
        annotations: vec![Annotation::AttrPacked],
        variant: TypeVariant::Builtin(BuiltinType::Int),
    };
    let err = compute_layout(Target::X86_64UnknownLinuxGnu, &ty).unwrap_err();
    assert!(matches!(err.kind(), ErrorType::AnnotatedBuiltinType));
}

#[test]
fn annotated_opaque() {
    let ty = Type::<()> {
        layout: (),
        annotations: vec![Annotation::AttrPacked],
        variant: TypeVariant::Opaque(TypeLayout {
            size_bits: 0,
            field_alignment_bits: 8,
            pointer_alignment_bits: 8,
            required_alignment_bits: 8
        }),
    };
    let err = compute_layout(Target::X86_64UnknownLinuxGnu, &ty).unwrap_err();
    assert!(matches!(err.kind(), ErrorType::AnnotatedOpaqueType));
}

#[test]
fn annotated_array() {
    let ty = Type::<()> {
        layout: (),
        annotations: vec![Annotation::AttrPacked],
        variant: TypeVariant::Array(Array {
            element_type: Box::new(Type {
                layout: (),
                annotations: vec![],
                variant: TypeVariant::Builtin(BuiltinType::Int),
            }),
            num_elements: None
        }),
    };
    let err = compute_layout(Target::X86_64UnknownLinuxGnu, &ty).unwrap_err();
    assert!(matches!(err.kind(), ErrorType::AnnotatedArray));
}

#[test]
fn size_overflow() {
    let ty = Type::<()> {
        layout: (),
        annotations: vec![],
        variant: TypeVariant::Array(Array {
            element_type: Box::new(Type {
                layout: (),
                annotations: vec![],
                variant: TypeVariant::Builtin(BuiltinType::Int),
            }),
            num_elements: Some(u64::MAX / 8 / 4 + 1),
        }),
    };
    let err = compute_layout(Target::X86_64UnknownLinuxGnu, &ty).unwrap_err();
    assert!(matches!(err.kind(), ErrorType::SizeOverflow));
}

#[test]
fn power_of_two_alignment_1() {
    let ty = Type::<()> {
        layout: (),
        annotations: vec![Annotation::Align(Some(24))],
        variant: TypeVariant::Record(Record {
            kind: RecordKind::Struct,
            fields: vec![]
        }),
    };
    let err = compute_layout(Target::X86_64UnknownLinuxGnu, &ty).unwrap_err();
    assert!(matches!(err.kind(), ErrorType::PowerOfTwoAlignment));
}

#[test]
fn power_of_two_alignment_2() {
    let ty = Type::<()> {
        layout: (),
        annotations: vec![],
        variant: TypeVariant::Opaque(TypeLayout {
            size_bits: 0,
            field_alignment_bits: 8,
            pointer_alignment_bits: 24,
            required_alignment_bits: 8
        }),
    };
    let err = compute_layout(Target::X86_64UnknownLinuxGnu, &ty).unwrap_err();
    assert!(matches!(err.kind(), ErrorType::PowerOfTwoAlignment));
}

#[test]
fn power_of_two_alignment_3() {
    let ty = Type::<()> {
        layout: (),
        annotations: vec![],
        variant: TypeVariant::Opaque(TypeLayout {
            size_bits: 0,
            field_alignment_bits: 24,
            pointer_alignment_bits: 8,
            required_alignment_bits: 8
        }),
    };
    let err = compute_layout(Target::X86_64UnknownLinuxGnu, &ty).unwrap_err();
    assert!(matches!(err.kind(), ErrorType::PowerOfTwoAlignment));
}

#[test]
fn power_of_two_alignment_4() {
    let ty = Type::<()> {
        layout: (),
        annotations: vec![],
        variant: TypeVariant::Opaque(TypeLayout {
            size_bits: 0,
            field_alignment_bits: 8,
            pointer_alignment_bits: 8,
            required_alignment_bits: 24
        }),
    };
    let err = compute_layout(Target::X86_64UnknownLinuxGnu, &ty).unwrap_err();
    assert!(matches!(err.kind(), ErrorType::PowerOfTwoAlignment));
}




#[test]
fn sub_byte_alignment_1() {
    let ty = Type::<()> {
        layout: (),
        annotations: vec![Annotation::Align(Some(4))],
        variant: TypeVariant::Record(Record {
            kind: RecordKind::Struct,
            fields: vec![]
        }),
    };
    let err = compute_layout(Target::X86_64UnknownLinuxGnu, &ty).unwrap_err();
    assert!(matches!(err.kind(), ErrorType::SubByteAlignment));
}

#[test]
fn sub_byte_alignment_2() {
    let ty = Type::<()> {
        layout: (),
        annotations: vec![],
        variant: TypeVariant::Opaque(TypeLayout {
            size_bits: 0,
            field_alignment_bits: 8,
            pointer_alignment_bits: 4,
            required_alignment_bits: 8
        }),
    };
    let err = compute_layout(Target::X86_64UnknownLinuxGnu, &ty).unwrap_err();
    assert!(matches!(err.kind(), ErrorType::SubByteAlignment));
}

#[test]
fn sub_byte_alignment_3() {
    let ty = Type::<()> {
        layout: (),
        annotations: vec![],
        variant: TypeVariant::Opaque(TypeLayout {
            size_bits: 0,
            field_alignment_bits: 4,
            pointer_alignment_bits: 8,
            required_alignment_bits: 8
        }),
    };
    let err = compute_layout(Target::X86_64UnknownLinuxGnu, &ty).unwrap_err();
    assert!(matches!(err.kind(), ErrorType::SubByteAlignment));
}

#[test]
fn sub_byte_alignment_4() {
    let ty = Type::<()> {
        layout: (),
        annotations: vec![],
        variant: TypeVariant::Opaque(TypeLayout {
            size_bits: 0,
            field_alignment_bits: 8,
            pointer_alignment_bits: 8,
            required_alignment_bits: 4
        }),
    };
    let err = compute_layout(Target::X86_64UnknownLinuxGnu, &ty).unwrap_err();
    assert!(matches!(err.kind(), ErrorType::SubByteAlignment));
}

#[test]
fn sub_byte_size() {
    let ty = Type::<()> {
        layout: (),
        annotations: vec![],
        variant: TypeVariant::Opaque(TypeLayout {
            size_bits: 4,
            field_alignment_bits: 8,
            pointer_alignment_bits: 8,
            required_alignment_bits: 8,
        }),
    };
    let err = compute_layout(Target::X86_64UnknownLinuxGnu, &ty).unwrap_err();
    assert!(matches!(err.kind(), ErrorType::SubByteSize));
}

#[test]
fn multiple_pragma_pack() {
    let ty = Type::<()> {
        layout: (),
        annotations: vec![Annotation::PragmaPack(8), Annotation::PragmaPack(8)],
        variant: TypeVariant::Record(Record {
            kind: RecordKind::Struct,
            fields: vec![]
        }),
    };
    let err = compute_layout(Target::X86_64UnknownLinuxGnu, &ty).unwrap_err();
    assert!(matches!(err.kind(), ErrorType::MultiplePragmaPackedAnnotations));
}

#[test]
fn named_zero_sized_bit_field() {
    let ty = Type::<()> {
        layout: (),
        annotations: vec![],
        variant: TypeVariant::Record(Record {
            kind: RecordKind::Struct,
            fields: vec![
                RecordField {
                    layout: None,
                    annotations: vec![],
                    named: true,
                    bit_width: Some(0),
                    ty: Type {
                        layout: (),
                        annotations: vec![],
                        variant: TypeVariant::Builtin(BuiltinType::Int),
                    }
                }
            ]
        }),
    };
    let err = compute_layout(Target::X86_64UnknownLinuxGnu, &ty).unwrap_err();
    assert!(matches!(err.kind(), ErrorType::NamedZeroSizeBitField));
}

#[test]
fn unnamed_regular_field() {
    let ty = Type::<()> {
        layout: (),
        annotations: vec![],
        variant: TypeVariant::Record(Record {
            kind: RecordKind::Struct,
            fields: vec![
                RecordField {
                    layout: None,
                    annotations: vec![],
                    named: false,
                    bit_width: None,
                    ty: Type {
                        layout: (),
                        annotations: vec![],
                        variant: TypeVariant::Builtin(BuiltinType::Int),
                    }
                }
            ]
        }),
    };
    let err = compute_layout(Target::X86_64UnknownLinuxGnu, &ty).unwrap_err();
    assert!(matches!(err.kind(), ErrorType::UnnamedRegularField));
}

#[test]
fn oversized_bitfield() {
    let ty = Type::<()> {
        layout: (),
        annotations: vec![],
        variant: TypeVariant::Record(Record {
            kind: RecordKind::Struct,
            fields: vec![
                RecordField {
                    layout: None,
                    annotations: vec![],
                    named: true,
                    bit_width: Some(64),
                    ty: Type {
                        layout: (),
                        annotations: vec![],
                        variant: TypeVariant::Builtin(BuiltinType::Int),
                    }
                }
            ]
        }),
    };
    let err = compute_layout(Target::X86_64UnknownLinuxGnu, &ty).unwrap_err();
    assert!(matches!(err.kind(), ErrorType::OversizedBitfield));
}

#[test]
fn pragma_packed_field() {
    let ty = Type::<()> {
        layout: (),
        annotations: vec![],
        variant: TypeVariant::Record(Record {
            kind: RecordKind::Struct,
            fields: vec![
                RecordField {
                    layout: None,
                    annotations: vec![Annotation::PragmaPack(8)],
                    named: true,
                    bit_width: None,
                    ty: Type {
                        layout: (),
                        annotations: vec![],
                        variant: TypeVariant::Builtin(BuiltinType::Int),
                    }
                }
            ]
        }),
    };
    let err = compute_layout(Target::X86_64UnknownLinuxGnu, &ty).unwrap_err();
    assert!(matches!(err.kind(), ErrorType::PragmaPackedField));
}
