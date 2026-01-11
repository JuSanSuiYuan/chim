use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnumId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TraitId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VarId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstId(usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructField {
    pub name: Arc<str>,
    pub ty: TypeId,
    pub offset: usize,
    pub size: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructData {
    pub name: Arc<str>,
    pub fields: Vec<StructField>,
    pub size: usize,
    pub align: usize,
    pub is_packed: bool,
    pub is_pub: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumVariant {
    pub name: Arc<str>,
    pub discriminant: i128,
    pub fields: Vec<StructField>,
    pub size: usize,
    pub align: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumData {
    pub name: Arc<str>,
    pub variants: Vec<EnumVariant>,
    pub size: usize,
    pub align: usize,
    pub is_c_like: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TagRepresentation {
    U8,
    U16,
    U32,
    U64,
    Usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeData {
    Never,
    Unit,
    Bool,
    Char,
    Int(IntSize),
    Uint(UintSize),
    Float(FloatSize),
    Str,
    Bytes,
    Pointer(TypeId, Mutability),
    Reference(TypeId, Option<LifetimeId>, Mutability),
    Array(TypeId, usize),
    Slice(TypeId),
    Tuple(Vec<TypeId>),
    Function(FunctionId),
    Closure,
    Generator,
    Struct(StructId),
    Enum(EnumId),
    TraitObject(TraitId),
    Alias { name: Arc<str>, target: TypeId },
    Parametric { name: Arc<str>, index: usize },
    Error,
    Infer(InferKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntSize {
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UintSize {
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FloatSize {
    F32,
    F64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mutability {
    Mutable,
    Immutable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InferKind {
    Type,
    Int,
    Float,
    Array,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LifetimeId(usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LifetimeData {
    pub name: Arc<str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionSig {
    pub params: Vec<TypeId>,
    pub return_type: TypeId,
    pub is_async: bool,
}

impl Hash for FunctionSig {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.params.len().hash(state);
        for &param in &self.params {
            param.hash(state);
        }
        self.return_type.hash(state);
        self.is_async.hash(state);
    }
}

impl PartialEq for FunctionSig {
    fn eq(&self, other: &Self) -> bool {
        self.params == other.params
            && self.return_type == other.return_type
            && self.is_async == other.is_async
    }
}

impl Eq for FunctionSig {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionData {
    pub name: Arc<str>,
    pub sig: FunctionSig,
    pub is_generic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraitData {
    pub name: Arc<str>,
    pub super_traits: Vec<TraitId>,
    pub methods: Vec<FunctionSig>,
}

#[derive(Debug, Clone, Default)]
pub struct TypePool {
    types: Vec<TypeData>,
    structs: Vec<StructData>,
    enums: Vec<EnumData>,
    traits: Vec<TraitData>,
    functions: Vec<FunctionData>,
    builtin_types: BuiltinTypes,
}

#[derive(Debug, Clone)]
pub struct BuiltinTypes {
    pub never: TypeId,
    pub unit: TypeId,
    pub bool: TypeId,
    pub char: TypeId,
    pub i8: TypeId,
    pub i16: TypeId,
    pub i32: TypeId,
    pub i64: TypeId,
    pub i128: TypeId,
    pub isize: TypeId,
    pub u8: TypeId,
    pub u16: TypeId,
    pub u32: TypeId,
    pub u64: TypeId,
    pub u128: TypeId,
    pub usize: TypeId,
    pub f32: TypeId,
    pub f64: TypeId,
    pub str: TypeId,
    pub bytes: TypeId,
}

impl TypePool {
    pub fn new() -> Self {
        let mut pool = TypePool::default();
        pool.init_builtin_types();
        pool
    }

    fn init_builtin_types(&mut self) {
        self.builtin_types.never = self.intern_type(TypeData::Never);
        self.builtin_types.unit = self.intern_type(TypeData::Unit);
        self.builtin_types.bool = self.intern_type(TypeData::Bool);
        self.builtin_types.char = self.intern_type(TypeData::Char);
        self.builtin_types.i8 = self.intern_type(TypeData::Int(IntSize::I8));
        self.builtin_types.i16 = self.intern_type(TypeData::Int(IntSize::I16));
        self.builtin_types.i32 = self.intern_type(TypeData::Int(IntSize::I32));
        self.builtin_types.i64 = self.intern_type(TypeData::Int(IntSize::I64));
        self.builtin_types.i128 = self.intern_type(TypeData::Int(IntSize::I128));
        self.builtin_types.isize = self.intern_type(TypeData::Int(IntSize::Isize));
        self.builtin_types.u8 = self.intern_type(TypeData::Uint(UintSize::U8));
        self.builtin_types.u16 = self.intern_type(TypeData::Uint(UintSize::U16));
        self.builtin_types.u32 = self.intern_type(TypeData::Uint(UintSize::U32));
        self.builtin_types.u64 = self.intern_type(TypeData::Uint(UintSize::U64));
        self.builtin_types.u128 = self.intern_type(TypeData::Uint(UintSize::U128));
        self.builtin_types.usize = self.intern_type(TypeData::Uint(UintSize::Usize));
        self.builtin_types.f32 = self.intern_type(TypeData::Float(FloatSize::F32));
        self.builtin_types.f64 = self.intern_type(TypeData::Float(FloatSize::F64));
        self.builtin_types.str = self.intern_type(TypeData::Str);
        self.builtin_types.bytes = self.intern_type(TypeData::Bytes);
    }

    pub fn intern_type(&mut self, ty: TypeData) -> TypeId {
        if let Some(idx) = self.types.iter().position(|t| t == &ty) {
            return TypeId(idx);
        }
        let id = TypeId(self.types.len());
        self.types.push(ty);
        id
    }

    pub fn get_type(&self, id: TypeId) -> &TypeData {
        self.types.get(id.0).unwrap_or(&TypeData::Error)
    }

    pub fn get_struct(&self, id: StructId) -> &StructData {
        self.structs.get(id.0).unwrap_or_else(|| panic!("struct {} not found", id.0))
    }

    pub fn get_enum(&self, id: EnumId) -> &EnumData {
        self.enums.get(id.0).unwrap_or_else(|| panic!("enum {} not found", id.0))
    }

    pub fn get_function(&self, id: FunctionId) -> &FunctionData {
        self.functions.get(id.0).unwrap_or_else(|| panic!("function {} not found", id.0))
    }

    pub fn type_count(&self) -> usize {
        self.types.len()
    }

    pub fn struct_count(&self) -> usize {
        self.structs.len()
    }

    pub fn enum_count(&self) -> usize {
        self.enums.len()
    }

    pub fn function_count(&self) -> usize {
        self.functions.len()
    }

    pub fn type_size(&self, ty_id: TypeId) -> usize {
        match self.get_type(ty_id) {
            TypeData::Never => 0,
            TypeData::Unit => 0,
            TypeData::Bool => 1,
            TypeData::Char => 4,
            TypeData::Int(size) => match size {
                IntSize::I8 => 1,
                IntSize::I16 => 2,
                IntSize::I32 => 4,
                IntSize::I64 => 8,
                IntSize::I128 => 16,
                IntSize::Isize => std::mem::size_of::<isize>(),
            },
            TypeData::Uint(size) => match size {
                UintSize::U8 => 1,
                UintSize::U16 => 2,
                UintSize::U32 => 4,
                UintSize::U64 => 8,
                UintSize::U128 => 16,
                UintSize::Usize => std::mem::size_of::<usize>(),
            },
            TypeData::Float(size) => match size {
                FloatSize::F32 => 4,
                FloatSize::F64 => 8,
            },
            TypeData::Str => std::mem::size_of::<&str>(),
            TypeData::Bytes => std::mem::size_of::<&[u8]>(),
            TypeData::Pointer(_, _) | TypeData::Reference(_, _, _) => std::mem::size_of::<usize>(),
            TypeData::Array(elem_ty, len) => self.type_size(*elem_ty) * len,
            TypeData::Slice(elem_ty) => std::mem::size_of::<&[T]>().max(self.type_size(*elem_ty)),
            TypeData::Tuple(elems) => elems.iter().map(|ty| self.type_size(*ty)).sum(),
            TypeData::Struct(struct_id) => {
                if let Some(struct_data) = self.structs.get(struct_id.0) {
                    struct_data.size
                } else {
                    0
                }
            }
            TypeData::Enum(enum_id) => {
                if let Some(enum_data) = self.enums.get(enum_id.0) {
                    enum_data.size
                } else {
                    0
                }
            }
            TypeData::Function(_) | TypeData::Closure | TypeData::Generator => {
                std::mem::size_of::<usize>()
            }
            TypeData::TraitObject(_) => std::mem::size_of::<usize>() * 2,
            TypeData::Alias { target, .. } => self.type_size(*target),
            TypeData::Parametric { .. } => std::mem::size_of::<usize>(),
            TypeData::Error => 0,
            TypeData::Infer(_) => std::mem::size_of::<usize>(),
        }
    }

    pub fn type_align(&self, ty_id: TypeId) -> usize {
        match self.get_type(ty_id) {
            TypeData::Never => 1,
            TypeData::Unit => 1,
            TypeData::Bool => 1,
            TypeData::Char => 4,
            TypeData::Int(size) => match size {
                IntSize::I8 => 1,
                IntSize::I16 => 2,
                IntSize::I32 => 4,
                IntSize::I64 => 8,
                IntSize::I128 => 16,
                IntSize::Isize => std::mem::align_of::<isize>(),
            },
            TypeData::Uint(size) => match size {
                UintSize::U8 => 1,
                UintSize::U16 => 2,
                UintSize::U32 => 4,
                UintSize::U64 => 8,
                UintSize::U128 => 16,
                UintSize::Usize => std::mem::align_of::<usize>(),
            },
            TypeData::Float(size) => match size {
                FloatSize::F32 => 4,
                FloatSize::F64 => 8,
            },
            TypeData::Str => std::mem::align_of::<&str>(),
            TypeData::Bytes => std::mem::align_of::<&[u8]>(),
            TypeData::Pointer(_, _) | TypeData::Reference(_, _, _) => std::mem::align_of::<usize>(),
            TypeData::Array(elem_ty, _) => self.type_align(*elem_ty),
            TypeData::Slice(elem_ty) => self.type_align(*elem_ty).max(std::mem::align_of::<usize>()),
            TypeData::Tuple(elems) => elems.iter().map(|ty| self.type_align(*ty)).max().unwrap_or(1),
            TypeData::Struct(struct_id) => {
                if let Some(struct_data) = self.structs.get(struct_id.0) {
                    struct_data.align
                } else {
                    1
                }
            }
            TypeData::Enum(enum_id) => {
                if let Some(enum_data) = self.enums.get(enum_id.0) {
                    enum_data.align
                } else {
                    1
                }
            }
            TypeData::Function(_) | TypeData::Closure | TypeData::Generator => {
                std::mem::align_of::<usize>()
            }
            TypeData::TraitObject(_) => std::mem::align_of::<usize>(),
            TypeData::Alias { target, .. } => self.type_align(*target),
            TypeData::Parametric { .. } => std::mem::align_of::<usize>(),
            TypeData::Error => 1,
            TypeData::Infer(_) => std::mem::align_of::<usize>(),
        }
    }

    pub fn is_stack_allocated(&self, ty_id: TypeId) -> bool {
        match self.get_type(ty_id) {
            TypeData::Never | TypeData::Unit | TypeData::Bool | TypeData::Char => true,
            TypeData::Int(_) | TypeData::Uint(_) | TypeData::Float(_) => true,
            TypeData::Array(elem_ty, len) => {
                len > 0 && self.type_size(*elem_ty) * len <= 2048
            }
            TypeData::Tuple(elems) => {
                self.type_size(ty_id) <= 2048
            }
            TypeData::Struct(struct_id) => {
                if let Some(struct_data) = self.structs.get(struct_id.0) {
                    struct_data.size <= 2048
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn is_heap_allocated(&self, ty_id: TypeId) -> bool {
        !self.is_stack_allocated(ty_id)
    }
}

impl fmt::Display for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TypeId({})", self.0)
    }
}

impl fmt::Display for TypeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeData::Never => write!(f, "!"),
            TypeData::Unit => write!(f, "()"),
            TypeData::Bool => write!(f, "bool"),
            TypeData::Char => write!(f, "char"),
            TypeData::Int(size) => write!(f, "{}", size),
            TypeData::Uint(size) => write!(f, "{}", size),
            TypeData::Float(size) => write!(f, "{}", size),
            TypeData::Str => write!(f, "str"),
            TypeData::Bytes => write!(f, "[u8]"),
            TypeData::Pointer(ty, _) => write!(f, "*{}", ty),
            TypeData::Reference(ty, _, _) => write!(f, "&{}", ty),
            TypeData::Array(ty, size) => write!(f, "[{}; {}]", ty, size),
            TypeData::Slice(ty) => write!(f, "[{}]", ty),
            TypeData::Tuple(types) => {
                write!(f, "({})", types.iter().map(|t| format!("{}", t)).collect::<Vec<_>>().join(", "))
            }
            TypeData::Function(func_id) => write!(f, "fn_{}", func_id.0),
            TypeData::Closure => write!(f, "closure"),
            TypeData::Generator => write!(f, "generator"),
            TypeData::Struct(id) => write!(f, "struct_{}", id.0),
            TypeData::Enum(id) => write!(f, "enum_{}", id.0),
            TypeData::TraitObject(id) => write!(f, "dyn trait_{}", id.0),
            TypeData::Alias { name, .. } => write!(f, "{}", name),
            TypeData::Parametric { name, index } => write!(f, "${}", index),
            TypeData::Error => write!(f, "{{error}}"),
            TypeData::Infer(kind) => write!(f, "{{infer: {:?}}}", kind),
        }
    }
}

impl fmt::Display for IntSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntSize::I8 => write!(f, "i8"),
            IntSize::I16 => write!(f, "i16"),
            IntSize::I32 => write!(f, "i32"),
            IntSize::I64 => write!(f, "i64"),
            IntSize::I128 => write!(f, "i128"),
            IntSize::Isize => write!(f, "isize"),
        }
    }
}

impl fmt::Display for UintSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UintSize::U8 => write!(f, "u8"),
            UintSize::U16 => write!(f, "u16"),
            UintSize::U32 => write!(f, "u32"),
            UintSize::U64 => write!(f, "u64"),
            UintSize::U128 => write!(f, "u128"),
            UintSize::Usize => write!(f, "usize"),
        }
    }
}

impl fmt::Display for FloatSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FloatSize::F32 => write!(f, "f32"),
            FloatSize::F64 => write!(f, "f64"),
        }
    }
}

impl fmt::Display for Mutability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mutability::Mutable => write!(f, "mut"),
            Mutability::Immutable => write!(f, "const"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_interning() {
        let mut pool = TypePool::new();

        let int1 = pool.intern_type(TypeData::Int(IntSize::I32));
        let int2 = pool.intern_type(TypeData::Int(IntSize::I32));
        let float = pool.intern_type(TypeData::Float(FloatSize::F64));

        assert_eq!(int1, int2);
        assert_ne!(int1, float);
    }

    #[test]
    fn test_struct_creation() {
        let mut pool = TypePool::new();

        let int_type = pool.intern_type(TypeData::Int(IntSize::I32));
        let fields = vec![
            StructField {
                name: Arc::from("x"),
                ty: int_type,
                offset: 0,
                size: 4,
            },
            StructField {
                name: Arc::from("y"),
                ty: int_type,
                offset: 4,
                size: 4,
            },
        ];

        let struct_data = StructData {
            name: Arc::from("Point"),
            fields,
            size: 8,
            align: 4,
            is_packed: false,
            is_pub: true,
        };

        assert_eq!(struct_data.name.as_ref(), "Point");
        assert_eq!(struct_data.fields.len(), 2);
    }
}
