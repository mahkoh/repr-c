// SPDX-License-Identifier: MIT OR Apache-2.0
//! This crate contains APIs that allow you to calculate the layout of C types.
//!
//! # Example
//!
//! Consider the C type
//!
//! ```c
//! struct __attribute__((packed)) X {
//!     char c;
//!     int i:2 __attribute__((aligned(16)));
//! };
//! ```
//!
//! You can compute the layout of this type as follows:
//!
//! ```rust
//! # use repc::layout::{Type, Annotation, Record, RecordKind, RecordField, TypeVariant, BuiltinType, TypeLayout, FieldLayout};
//! # use repc::{compute_layout, Target};
//! let ty = Type::<()> {
//!     layout: (),
//!     annotations: vec![Annotation::AttrPacked],
//!     variant: TypeVariant::Record(Record {
//!         kind: RecordKind::Struct,
//!         fields: vec![
//!             RecordField {
//!                 layout: None,
//!                 annotations: vec![],
//!                 named: true,
//!                 bit_width: None,
//!                 ty: Type {
//!                     layout: (),
//!                     annotations: vec![],
//!                     variant: TypeVariant::Builtin(BuiltinType::Char),
//!                 },
//!             },
//!             RecordField {
//!                 layout: None,
//!                 annotations: vec![Annotation::Align(Some(128))],
//!                 named: true,
//!                 bit_width: Some(2),
//!                 ty: Type {
//!                     layout: (),
//!                     annotations: vec![],
//!                     variant: TypeVariant::Builtin(BuiltinType::Int),
//!                 },
//!             },
//!         ]
//!     }),
//! };
//! let layout = compute_layout(Target::X86_64UnknownLinuxGnu, &ty).unwrap();
//! assert_eq!(layout.layout, TypeLayout {
//!     size_bits: 256,
//!     field_alignment_bits: 128,
//!     pointer_alignment_bits: 128,
//!     required_alignment_bits: 8,
//! });
//! let fields = match &layout.variant {
//!     TypeVariant::Record(r) => &r.fields,
//!     _ => unreachable!(),
//! };
//! assert_eq!(fields[0].layout.unwrap(), FieldLayout {
//!     offset_bits: 0,
//!     size_bits: 8,
//! });
//! assert_eq!(fields[1].layout.unwrap(), FieldLayout {
//!     offset_bits: 128,
//!     size_bits: 2,
//! });
//! println!("{:#?}", layout);
//! ```

pub use repc_impl::builder::compute_layout;

pub use repc_impl::target::{Target, HOST_TARGET, TARGETS};

pub mod layout {
    //! Types describing the structure and layout of C types.
    //!
    //! Maybe of these types take a [`Layout`] type parameter. The two predominant implementations
    //! of `Layout` are `TypeLayout` and `()`. `Type<TypeLayout>` can be converted to `Type<()>`
    //! by calling `Type::<TypeLayout>::into()`.

    pub use repc_impl::layout::{
        Annotation, Array, BuiltinType, FieldLayout, Layout, Record, RecordField, RecordKind, Type,
        TypeLayout, TypeVariant,
    };
}

pub use repc_impl::result::{Error, ErrorType};

pub mod visitor {
    //! Types and functions allowing you to traverse a [`Type`](crate::layout::Type).

    pub use repc_impl::visitor::Visitor;

    pub use repc_impl::visitor::{
        visit_annotations, visit_array, visit_builtin_type, visit_enum, visit_opaque_type,
        visit_record, visit_record_field, visit_type, visit_typedef,
    };
}
