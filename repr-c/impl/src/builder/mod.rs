use std::ops::Not;

use crate::layout::{Annotation, Array, BuiltinType, Record, RecordField, Type, TypeLayout};
use crate::result::{Error, Result};
use crate::target::Target;
use crate::util::BITS_PER_BYTE;
use crate::visitor::{
    visit_array, visit_builtin_type, visit_opaque_type, visit_record_field, visit_typedef, Visitor,
};

pub mod common;
mod msvc;
mod sysv;

pub fn compute_layout(target: Target, ty: &Type<()>) -> Result<Type<TypeLayout>> {
    pre_validate(ty)?;
    use Target::*;
    let ty = match target {
        | I686PcWindowsGnu | X86_64PcWindowsGnu => sysv::mingw::compute_layout(target, ty),
        | Aarch64PcWindowsMsvc
        | I586PcWindowsMsvc
        | I686PcWindowsMsvc
        | I686UnknownWindows
        | Thumbv7aPcWindowsMsvc
        | X86_64UnknownWindows
        | X86_64PcWindowsMsvc => msvc::compute_layout(target, ty),
        | Aarch64Fuchsia
        | Aarch64LinuxAndroid
        | Aarch64UnknownFreebsd
        | Aarch64UnknownHermit
        | Aarch64UnknownLinuxGnu
        | Aarch64UnknownLinuxMusl
        | Aarch64UnknownNetbsd
        | Aarch64UnknownNone
        | Aarch64UnknownOpenbsd
        | Aarch64UnknownRedox
        | Arm64AppleIos
        | Arm64AppleIosMacabi
        | Arm64AppleTvos
        | Armebv7rUnknownNoneEabi
        | Armebv7rUnknownNoneEabihf
        | ArmLinuxAndroideabi
        | ArmUnknownLinuxGnueabi
        | ArmUnknownLinuxGnueabihf
        | Armv4tUnknownLinuxGnueabi
        | Armv5teUnknownLinuxGnueabi
        | Armv5teUnknownLinuxUclibcgnueabi
        | Armv6UnknownFreebsdGnueabihf
        | Armv6UnknownNetbsdelfEabihf
        | Armv7aNoneEabi
        | Armv7aNoneEabihf
        | Armv7AppleIos
        | Armv7NoneLinuxAndroid
        | Armv7rUnknownNoneEabi
        | Armv7rUnknownNoneEabihf
        | Armv7sAppleIos
        | Armv7UnknownFreebsdGnueabihf
        | Armv7UnknownLinuxGnueabi
        | Armv7UnknownLinuxGnueabihf
        | Armv7UnknownNetbsdelfEabihf
        | AvrUnknownUnknown
        | HexagonUnknownLinuxMusl
        | I386AppleIos
        | I586UnknownLinuxGnu
        | I586UnknownLinuxMusl
        | I686LinuxAndroid
        | I686UnknownFreebsd
        | I686UnknownHaiku
        | I686UnknownLinuxGnu
        | I686UnknownLinuxMusl
        | I686UnknownNetbsdelf
        | I686UnknownOpenbsd
        | Mips64elUnknownLinuxGnuabi64
        | Mips64elUnknownLinuxMusl
        | Mips64UnknownLinuxGnuabi64
        | Mips64UnknownLinuxMusl
        | MipselSonyPsp
        | MipselUnknownLinuxGnu
        | MipselUnknownLinuxMusl
        | MipselUnknownLinuxUclibc
        | MipselUnknownNone
        | Mipsisa32r6elUnknownLinuxGnu
        | Mipsisa32r6UnknownLinuxGnu
        | Mipsisa64r6elUnknownLinuxGnuabi64
        | Mipsisa64r6UnknownLinuxGnuabi64
        | MipsUnknownLinuxGnu
        | MipsUnknownLinuxMusl
        | MipsUnknownLinuxUclibc
        | Msp430NoneElf
        | Powerpc64leUnknownLinuxGnu
        | Powerpc64leUnknownLinuxMusl
        | Powerpc64UnknownFreebsd
        | Powerpc64UnknownLinuxGnu
        | Powerpc64UnknownLinuxMusl
        | PowerpcUnknownLinuxGnu
        | PowerpcUnknownLinuxGnuspe
        | PowerpcUnknownLinuxMusl
        | PowerpcUnknownNetbsd
        | Riscv32
        | Riscv32UnknownLinuxGnu
        | Riscv64
        | Riscv64UnknownLinuxGnu
        | S390xUnknownLinuxGnu
        | Sparc64UnknownLinuxGnu
        | Sparc64UnknownNetbsd
        | Sparc64UnknownOpenbsd
        | SparcUnknownLinuxGnu
        | Sparcv9SunSolaris
        | Thumbv4tNoneEabi
        | Thumbv6mNoneEabi
        | Thumbv7emNoneEabi
        | Thumbv7emNoneEabihf
        | Thumbv7mNoneEabi
        | Thumbv8mBaseNoneEabi
        | Thumbv8mMainNoneEabi
        | Thumbv8mMainNoneEabihf
        | Wasm32UnknownEmscripten
        | Wasm32UnknownUnknown
        | Wasm32Wasi
        | X86_64AppleIos
        | X86_64AppleIos13_0Macabi
        | X86_64AppleTvos
        | X86_64Elf
        | X86_64Fuchsia
        | X86_64LinuxAndroid
        | X86_64PcSolaris
        | X86_64RumprunNetbsd
        | X86_64UnknownDragonfly
        | X86_64UnknownFreebsd
        | X86_64UnknownHaiku
        | X86_64UnknownHermit
        | X86_64UnknownL4reUclibc
        | X86_64UnknownLinuxGnu
        | X86_64UnknownLinuxGnux32
        | X86_64UnknownLinuxMusl
        | X86_64UnknownNetbsd
        | X86_64UnknownOpenbsd
        | X86_64UnknownRedox => sysv::sysv::compute_layout(target, ty),
    }?;
    post_validate(&ty)?;
    Ok(ty)
}

fn pre_validate(ty: &Type<()>) -> Result<()> {
    let mut pv = PreValidator(vec![]);
    pv.visit_type(ty);
    match pv.0.pop() {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

fn post_validate(ty: &Type<TypeLayout>) -> Result<()> {
    let mut pv = PostValidator(vec![]);
    pv.visit_type(ty);
    match pv.0.pop() {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

struct PreValidator(Vec<Error>);

impl Visitor<()> for PreValidator {
    fn visit_annotations(&mut self, a: &[Annotation]) {
        let mut num_align = 0;
        let mut num_packed = 0;
        let mut num_pragma_packed = 0;
        for a in a {
            match a {
                Annotation::PragmaPack(n) => {
                    num_pragma_packed += 1;
                    self.validate_alignment(*n);
                }
                Annotation::AttrPacked => num_packed += 1,
                Annotation::Aligned(n) => {
                    num_align += 1;
                    self.validate_alignment(*n);
                }
            }
        }
        if num_align > 1 {
            self.0.push(Error::MultipleAlignmentAnnotations);
        }
        if num_packed > 1 || num_pragma_packed > 1 {
            self.0.push(Error::MultiplePackedAnnotations);
        }
    }

    fn visit_builtin_type(&mut self, bi: BuiltinType, ty: &Type<()>) {
        if ty.annotations.is_empty().not() {
            self.0.push(Error::AnnotatedBuiltinType);
        }
        visit_builtin_type(self, bi, ty);
    }

    fn visit_record_field(&mut self, field: &RecordField<()>, rt: &Record<()>, ty: &Type<()>) {
        match (field.bit_width, field.named) {
            (Some(0), true) => self.0.push(Error::NamedZeroSizeBitField),
            (None, false) => self.0.push(Error::UnnamedRegularField),
            _ => {}
        }
        visit_record_field(self, field, rt, ty);
    }

    fn visit_typedef(&mut self, dst: &Type<()>, ty: &Type<()>) {
        for a in &dst.annotations {
            match a {
                Annotation::Aligned(_) => {}
                Annotation::PragmaPack(_) => self.0.push(Error::PackedTypedef),
                Annotation::AttrPacked => self.0.push(Error::PackedTypedef),
            }
        }
        visit_typedef(self, dst, ty);
    }

    fn visit_array(&mut self, at: &Array<()>, ty: &Type<()>) {
        if ty.annotations.is_empty().not() {
            self.0.push(Error::AnnotatedArray);
        }
        visit_array(self, at, ty);
    }

    fn visit_opaque_type(&mut self, layout: TypeLayout, ty: &Type<()>) {
        if ty.annotations.is_empty().not() {
            self.0.push(Error::AnnotatedOpaqueType);
        }
        if layout.size_bits % BITS_PER_BYTE != 0 {
            self.0.push(Error::SubByteSize);
        }
        self.validate_alignment(layout.field_alignment_bits);
        self.validate_alignment(layout.required_alignment_bits);
        visit_opaque_type(self, layout, ty);
    }
}

impl PreValidator {
    fn validate_alignment(&mut self, a: u64) {
        if a < BITS_PER_BYTE {
            self.0.push(Error::SubByteAlignment);
        }
        if a.is_power_of_two().not() {
            self.0.push(Error::PowerOfTwoAlignment);
        }
    }
}

struct PostValidator(Vec<Error>);

impl Visitor<TypeLayout> for PostValidator {
    fn visit_record_field(
        &mut self,
        field: &RecordField<TypeLayout>,
        rt: &Record<TypeLayout>,
        ty: &Type<TypeLayout>,
    ) {
        if let Some(n) = field.bit_width {
            if n > field.ty.layout.size_bits {
                self.0.push(Error::OversizedBitfield);
            }
        }
        visit_record_field(self, field, rt, ty);
    }
}
