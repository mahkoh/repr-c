use crate::layout::{BuiltinType, Type, TypeLayout, TypeVariant};
use crate::result::Result;
use crate::target::Target;

pub fn compute_builtin_type_layout(
    target: &dyn Target,
    bi: BuiltinType,
) -> Result<Type<TypeLayout>> {
    Ok(Type {
        layout: target.builtin_type_layout(bi),
        // Pre-validation ensures that builtin types do not have annotations.
        annotations: vec![],
        variant: TypeVariant::Builtin(bi),
    })
}

pub fn compute_opaque_type_layout(layout: TypeLayout) -> Result<Type<TypeLayout>> {
    Ok(Type {
        layout,
        // Pre-validation ensures that opaque types do not have annotations.
        annotations: vec![],
        variant: TypeVariant::Opaque(layout),
    })
}
