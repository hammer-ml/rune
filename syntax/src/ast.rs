//! The *Abstract Syntax Tree* for a Runefile.

use codespan::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Runefile {
    pub instructions: Vec<Instruction>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    From(FromInstruction),
    Model(ModelInstruction),
    Capability(CapabilityInstruction),
    Run(RunInstruction),
    ProcBlock(ProcBlockInstruction),
    Out(OutInstruction),
}

impl Instruction {
    pub fn span(&self) -> Span {
        match self {
            Instruction::From(f) => f.span,
            Instruction::Model(m) => m.span,
            Instruction::Capability(c) => c.span,
            Instruction::Run(r) => r.span,
            Instruction::ProcBlock(p) => p.span,
            Instruction::Out(o) => o.span,
        }
    }
}

impl From<FromInstruction> for Instruction {
    fn from(other: FromInstruction) -> Self { Instruction::From(other) }
}

impl From<ModelInstruction> for Instruction {
    fn from(other: ModelInstruction) -> Self { Instruction::Model(other) }
}

impl From<CapabilityInstruction> for Instruction {
    fn from(other: CapabilityInstruction) -> Self {
        Instruction::Capability(other)
    }
}

impl From<RunInstruction> for Instruction {
    fn from(other: RunInstruction) -> Self { Instruction::Run(other) }
}

impl From<ProcBlockInstruction> for Instruction {
    fn from(other: ProcBlockInstruction) -> Self {
        Instruction::ProcBlock(other)
    }
}

impl From<OutInstruction> for Instruction {
    fn from(other: OutInstruction) -> Self { Instruction::Out(other) }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FromInstruction {
    pub image: Path,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ModelInstruction {
    pub name: Ident,
    pub file: String,
    pub input_type: Type,
    pub output_type: Type,
    pub parameters: Vec<Argument>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    pub value: String,
    pub span: Span,
}

impl Ident {
    pub fn new(value: impl Into<String>, span: Span) -> Self {
        Ident {
            value: value.into(),
            span,
        }
    }

    /// Create an [`Ident`] with a placeholder span.
    #[cfg(test)]
    pub(crate) fn dangling(value: impl Into<String>) -> Self {
        Ident::new(value, Span::new(0, 0))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct CapabilityInstruction {
    pub kind: Ident,
    pub name: Ident,
    pub output_type: Type,
    pub parameters: Vec<Argument>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub kind: TypeKind,
    pub span: Span,
}

#[cfg(test)]
impl Type {
    pub(crate) fn named_dangling(name: impl Into<String>) -> Self {
        Type {
            kind: TypeKind::Named(Ident::dangling(name)),
            span: Span::new(0, 0),
        }
    }

    pub(crate) fn inferred_dangling() -> Self {
        Type {
            kind: TypeKind::Inferred,
            span: Span::new(0, 0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    Inferred,
    Named(Ident),
    Buffer {
        type_name: Ident,
        dimensions: Vec<usize>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct RunInstruction {
    pub steps: Vec<Ident>,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ProcBlockInstruction {
    pub path: Path,
    pub input_type: Type,
    pub output_type: Type,
    pub name: Ident,
    pub params: Vec<Argument>,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub struct OutInstruction {
    pub out_type: Ident,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Literal {
    pub kind: LiteralKind,
    pub span: Span,
}

impl Literal {
    pub fn new(kind: impl Into<LiteralKind>, span: Span) -> Self {
        Literal {
            kind: kind.into(),
            span,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum LiteralKind {
    Integer(i64),
    Float(f64),
    String(String),
}

impl From<i64> for LiteralKind {
    fn from(other: i64) -> Self { LiteralKind::Integer(other) }
}

impl From<f64> for LiteralKind {
    fn from(other: f64) -> Self { LiteralKind::Float(other) }
}

impl<'a> From<&'a str> for LiteralKind {
    fn from(other: &'a str) -> Self { LiteralKind::String(other.to_string()) }
}

impl From<String> for LiteralKind {
    fn from(other: String) -> Self { LiteralKind::String(other) }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Argument {
    pub name: Ident,
    pub value: ArgumentValue,
    pub span: Span,
}

impl Argument {
    pub fn literal(name: Ident, value: Literal, span: Span) -> Self {
        Argument {
            name,
            value: ArgumentValue::Literal(value),
            span,
        }
    }

    pub fn list<I>(name: Ident, values: I, span: Span) -> Self
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        Argument {
            name,
            value: ArgumentValue::List(
                values.into_iter().map(Into::into).collect(),
            ),
            span,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ArgumentValue {
    Literal(Literal),
    List(Vec<String>),
}

/// A specification for finding a dependency.
///
/// The full syntax is `base@version#sub_path` where
///
/// - `base` is a URL or the name of a repository on GitHub (e.g. `hotg-ai/rune`
///   or `https://github.com/hotg-ai/rune`)
/// - `version` is an optional field specifying the version (e.g. as a git tag)
/// - `sub_path` is an optional field which is useful when pointing to
///   repositories with multiple relevant items because it lets you specify
///   which directory the specified item is in.
#[derive(Debug, PartialEq, Clone)]
pub struct Path {
    pub base: String,
    pub sub_path: Option<String>,
    pub version: Option<String>,
    pub span: Span,
}

impl Path {
    pub fn new(
        base: impl Into<String>,
        sub_path: impl Into<Option<String>>,
        version: impl Into<Option<String>>,
        span: Span,
    ) -> Self {
        Path {
            base: base.into(),
            sub_path: sub_path.into(),
            version: version.into(),
            span,
        }
    }
}