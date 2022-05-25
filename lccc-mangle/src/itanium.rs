use std::{collections::HashMap, ops::Range};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Encoding {
    Function(Name, BareFunctionType),
    Data(Name),
    SpecialName(SpecialName),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Name {
    Nested(NestedName),
    Unscoped(UnscopedName),
    UnscopedTemplate(UnscopedTemplateName, Vec<TemplateArg>),
    Local(Box<LocalName>),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct LocalName {
    pub base: BaseEncoding,
    pub name: Option<Name>,
    pub discrim: Option<u64>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum BaseEncoding {
    Function(Encoding),
    Data(Encoding),
    Type(Encoding),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Sub {
    Numbered(Option<u64>),
    St,
    Sa,
    Sb,
    Ss,
    Si,
    So,
    Sd,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum UnscopedTemplateName {
    Name(UnscopedName),
    Sub(Sub),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum UnscopedName {
    Global(UnqualifiedName),
    // St3foo
    Std(UnqualifiedName),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum UnqualifiedName {
    OperatorName(Operator, Option<Vec<String>>),
    CtorDtor(CtorDtor),
    Name(String),
    UnamedType(Option<u64>),
    StructuredBinding(Vec<String>),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum CtorDtor {
    CompleteCtor,
    BaseCtor,
    CompleteAllocatingCtor,
    CompleteInheritingCtor(Box<Type>),
    BaseInheritingCtor(Box<Type>),
    DeletingDtor,
    CompleteDtor,
    BaseDtor,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Prefix {
    Name(Option<Box<Prefix>>, UnqualifiedName),
    Template(TemplatePrefix, Vec<TemplateArg>),
    TemplateParam(Option<u64>),
    Substitution(Sub),
}
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum TemplatePrefix {
    TemplateName(Option<Box<Prefix>>, UnqualifiedName),
    TemplateParam(Option<i64>),
    Substitution(Sub),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum TemplateArg {
    Type(Type),
    Expr(Expr),
    ArgumentPack(Vec<TemplateArg>),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum NestedName {
    Name(Prefix, UnqualifiedName),
    Template(TemplatePrefix, Vec<TemplateArg>),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Operator {
    New,
    NewArray,
    Delete,
    DeleteArray,
    Await,
    Pos,
    Neg,
    AddrOf,
    Deref,
    Compl,
    Plus,
    Minus,
    Multiply,
    Divide,
    Rem,
    And,
    Or,
    Eor,
    Assign,
    PlusAssign,
    MinusAssign,
    MultiplyAssign,
    DivideAssign,
    RemAssign,
    AndAssign,
    OrAssign,
    EorAssign,
    LeftShift,
    LeftShifAssign,
    RightShift,
    RightShiftAssign,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    Spaceship,
    Not,
    BoolAnd,
    BoolOr,
    Inc,
    Dec,
    Comma,
    PointerToMember,
    PointerMember,
    Parethesis,
    Index,
    Question,
    Cast(Box<Type>),
    Literal(String),
    VendorOperator(u8, String),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum SpecialName {
    VTable(Type),
    VTT(Type),
    TypeInfo(Type),
    TypeInfoName(Type),
    Thunk(Offset, Box<Encoding>),
    CovariantThunk {
        thisoff: Offset,
        retoff: Offset,
        base: Box<Encoding>,
    },
    Guard(Name),
    Temporary(Name, Option<u64>),
    TransactionSafeEntry(Box<Encoding>),
    TrackCallerThunk {
        fnname: Box<Encoding>,
        locname: Box<Encoding>,
        num: Option<u64>,
    },
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Offset {
    NonVirtual(i64),
    Virtual { thisof: i64, vcallof: i64 },
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Type {
    BuiltinType(BuiltinType),
    QualifiedType(Vec<Qualifier>, Box<Type>),
    FunctionType(FunctionType),
    Decltype(Box<Decltype>),
    ClassEnumType(ClassEnumType),
    ArrayType(ArrayType),
    PointerToMemberType { cl: Box<Type>, mem: Box<Type> },
    TemplateParam(Option<u64>),
    Pointer(Box<Type>),
    LValueRef(Box<Type>),
    RValueRef(Box<Type>),
    Complex(Box<Type>),
    Imaginary(Box<Type>),
    Substitution(Sub),
    PackExpansion(Box<Type>),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Decltype {
    Simple(UnresolvedName),
    Complex(Expr),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ArrayType {
    pub bound: ExprOrLit,
    pub element: Box<Type>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ClassEnumType {
    Name(Name),
    ElaboratedStruct(Name),
    ElaboratedUnion(Name),
    ElaboratedEnum(Name),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct BareFunctionType {
    pub sigtypes: Vec<Type>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct FunctionType {
    pub is_extern_c: bool,
    pub is_transaction_safe: bool,
    pub qualifiers: Vec<Qualifier>,
    pub refqualifier: Option<RefQualifier>,
    pub sig: BareFunctionType,
    pub noexcept: NoexceptSpec,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum RefQualifier {
    LValue,
    RValue,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum NoexceptSpec {
    Noexcept,
    CondNoexcept(Box<Expr>),
    DynamicSpec(Vec<Type>),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Qualifier {
    Const,
    Volatile,
    Restrict,
    VendorDefined(String, Vec<TemplateArg>),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum BuiltinType {
    Void,
    WCharT,
    Bool,
    Char,
    SChar,
    UChar,
    Short,
    UShort,
    Int,
    UInt,
    Long,
    ULong,
    LongLong,
    ULongLong,
    Int128,
    UInt128,
    Float,
    Double,
    LongDouble,
    Float128,
    Ellipsis,
    Decimal64,
    Decimal128,
    Decimal32,
    Half,
    BitInt(ExprOrLit),
    BitUint(ExprOrLit),
    FloatN(u16),
    Char32T,
    Char16T,
    Char8T,
    Auto,
    DecltypeAuto,
    NullptrT,
    Unit,
    Tuple(Vec<Type>),
    Slice(Box<Type>),
    UnboundLifetime,
    HTRB(u32, Box<Type>),
    VendorType(String, Vec<TemplateArg>),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ExprOrLit {
    Lit(i128),
    Expr(Box<Expr>),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Expr {
    Operator(Operator, Box<Expr>),
    PreInc(Box<Expr>),
    PreDec(Box<Expr>),
    Call(Vec<Expr>),
    ConvSingle(CastExpr),
    ConvList(Type, Vec<BracedExpr>),
    New(NewExpr),
    NewArray(NewExpr),
    Delete(DeleteExpr),
    DeleteArray(DeleteExpr),
    DynamicCast(CastExpr),
    StaticCast(CastExpr),
    ConstCast(CastExpr),
    ReinterpretCast(CastExpr),
    TypeId(Box<TypeOrExpr>),
    SizeOf(Box<TypeOrExpr>),
    AlignOf(Box<TypeOrExpr>),
    Noexcept(Box<Expr>),
    TemplateParam(Option<u64>),
    FunctionParam(FunctionParam),
    Dot(Box<Expr>, UnresolvedName),
    Arrow(Box<Expr>, UnresolvedName),
    DotDeref(Box<Expr>, Box<Expr>),
    SizeOfDotDotDot(PackRef),
    PackExpansion(Box<Expr>),
    LeftFold(Operator, Box<Expr>),
    RightFold(Box<Expr>, Operator),
    LeftFoldInit(Box<Expr>, Operator, Box<Expr>),
    RightFoldInit(Box<Expr>, Operator, Box<Expr>),
    Throw(Option<Box<Expr>>),
    UnitCtor,
    VendorDefined(String, Vec<TemplateArg>),
    Id(UnresolvedName),
    PrimaryExpr(PrimaryExpr),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum TypeOrExpr {
    Type(Type),
    Expr(Box<Expr>),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct CastExpr {
    pub ty: Type,
    pub expr: Box<Expr>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct NewExpr {
    pub gs: bool,
    pub placement_exprs: Vec<Expr>,
    pub ty: Type,
    pub init: Option<Initializer>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct DeleteExpr {
    pub gs: bool,
    pub target: Box<Expr>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Initializer {
    Paren(Vec<Expr>),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum FunctionParam {
    This,
    TopLevel(Vec<Qualifier>, Option<u64>),
    Nested {
        level: u64,
        qualifiers: Vec<Qualifier>,
        param_n: Option<u64>,
    },
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum PackRef {
    Template(Option<u64>),
    Function(FunctionParam),
    Captured(Vec<TemplateArg>),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum PrimaryExpr {
    IntLit(Type, i128),
    StringLit(Type),
    Nullptr,
    Null(Type),
    Complex(Type, i128, i128),
    ExternalName(Encoding),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum UnresolvedName {
    Unscoped(BaseUnresolvedName),
    Global(BaseUnresolvedName),
    Scoped {
        root: Option<ScopedNameRoot>,
        path: Vec<SimpleId>,
        terminal: BaseUnresolvedName,
    },
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ScopedNameRoot {
    GlobalScope,
    Type(UnresolvedType),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct SimpleId {
    pub name: String,
    pub template: Option<Vec<TemplateArg>>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum BaseUnresolvedName {
    Id(SimpleId),
    Operator(Operator, Option<Vec<TemplateArg>>),
    Destructor(DtorName),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum DtorName {
    Type(UnresolvedType),
    Id(SimpleId),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum UnresolvedType {
    TemplateParam(Option<u64>, Option<Vec<TemplateArg>>),
    Decltype(Box<Decltype>),
    Sub(Sub),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum BracedExpr {
    Bare(Expr),
    NamedField(String, Box<BracedExpr>),
    ArrayIndex(Expr, Box<BracedExpr>),
    ArrayRange(Range<Expr>, Box<BracedExpr>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SubstitutionMap {
    map: HashMap<Name, Option<u64>>,
    next: Option<u64>,
}

impl SubstitutionMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            next: None,
        }
    }

    pub fn substitute_name(&mut self, name: Name) -> Name {
        name
    }
}
