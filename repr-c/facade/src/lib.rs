pub use repr_c_impl::builder::compute_layout;

pub use repr_c_impl::target::{Target, HOST_TARGET, TARGETS};

pub mod layout {
    //! Types describing the structure and layout of C types.

    pub use repr_c_impl::layout::{
        Annotation, Array, BuiltinType, FieldLayout, Layout, Record, RecordField, RecordKind, Type,
        TypeLayout, TypeVariant,
    };
}

pub use repr_c_impl::result::{Error, ErrorKind};
