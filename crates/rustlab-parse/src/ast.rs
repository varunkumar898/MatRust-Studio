//! Abstract Syntax Tree (AST) definitions for MATLAB programs.

use crate::span::Span;

/// Binary operators supported in expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,        // +
    Sub,        // -
    Mul,        // *
    DotMul,     // .*
    Div,        // /
    DotDiv,     // ./
    LeftDiv,    // \
    DotLeftDiv, // .\
    Pow,        // ^
    DotPow,     // .^

    // Relational
    Eq,    // ==
    NotEq, // ~=
    Lt,    // <
    LtEq,  // <=
    Gt,    // >
    GtEq,  // >=

    // Logical
    And,    // &
    Or,     // |
    AndAnd, // &&
    OrOr,   // ||
}

/// Unary operators supported in expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Plus,               // +
    Minus,              // -
    Not,                // ~
    Transpose,          // .'
    ConjugateTranspose, // '
}

/// Expression node in the AST.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Numeric constant, e.g. `3.14159`
    Number(f64, Span),
    /// Imaginary constant, e.g. `3i`
    Imaginary(f64, Span),
    /// Character vector literal, e.g. `'hello'`
    CharVector(String, Span),
    /// String scalar literal, e.g. `"world"`
    StringLiteral(String, Span),
    /// Identifier or variable reference, e.g. `foo`
    Ident(String, Span),
    /// Dynamic `end` token inside an indexing expression, e.g. `A(2:end)`
    End(Span),
    /// Naked colon in slicing expressions, e.g. `A(:, 2)`
    Colon(Span),
    /// Colon range expression, e.g. `1:10` or `1:0.5:10`
    Range {
        start: Box<Expr>,
        step: Option<Box<Expr>>,
        end: Box<Expr>,
        span: Span,
    },
    /// Binary operator expression, e.g. `a + b`
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
    /// Unary operator expression, e.g. `-a`, `~b`, `A'`
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },
    /// Matrix literal constructor, e.g. `[1, 2; 3, 4]`
    Matrix { rows: Vec<Vec<Expr>>, span: Span },
    /// Cell array literal constructor, e.g. `{1, 'text'; 3, 4}`
    CellArray { rows: Vec<Vec<Expr>>, span: Span },
    /// Function call or array indexing, e.g. `f(x, y)` or `A(1, 2)`
    Index {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    /// Cell dereference indexing, e.g. `C{1, 2}`
    CellIndex {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    /// Static structure field access, e.g. `s.field`
    DotAccess {
        expr: Box<Expr>,
        field: String,
        span: Span,
    },
    /// Dynamic structure field access, e.g. `s.(var)`
    DynamicDotAccess {
        expr: Box<Expr>,
        field_expr: Box<Expr>,
        span: Span,
    },
    /// Named function handle, e.g. `@sin`
    FunctionHandle(String, Span),
    /// Anonymous function, e.g. `@(x, y) x + y`
    AnonymousFunction {
        params: Vec<String>,
        body: Box<Expr>,
        span: Span,
    },
}

impl Expr {
    /// Returns the source span of the expression.
    pub fn span(&self) -> Span {
        match self {
            Self::Number(_, span) => *span,
            Self::Imaginary(_, span) => *span,
            Self::CharVector(_, span) => *span,
            Self::StringLiteral(_, span) => *span,
            Self::Ident(_, span) => *span,
            Self::End(span) => *span,
            Self::Colon(span) => *span,
            Self::Range { span, .. } => *span,
            Self::Binary { span, .. } => *span,
            Self::Unary { span, .. } => *span,
            Self::Matrix { span, .. } => *span,
            Self::CellArray { span, .. } => *span,
            Self::Index { span, .. } => *span,
            Self::CellIndex { span, .. } => *span,
            Self::DotAccess { span, .. } => *span,
            Self::DynamicDotAccess { span, .. } => *span,
            Self::FunctionHandle(_, span) => *span,
            Self::AnonymousFunction { span, .. } => *span,
        }
    }
}

/// Target of an assignment statement.
#[derive(Debug, Clone, PartialEq)]
pub enum AssignTarget {
    /// Simple identifier assignment: `x = ...`
    Ident(String, Span),
    /// Subscripted assignment: `A(1, 2) = ...`
    Index {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    /// Cell element assignment: `C{1, 2} = ...`
    CellIndex {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    /// Field assignment: `s.field = ...`
    Dot {
        expr: Box<Expr>,
        field: String,
        span: Span,
    },
    /// Multiple variable assignment from function returns: `[a, b, c] = ...`
    MultiAssign(Vec<AssignTarget>, Span),
}

impl AssignTarget {
    pub fn span(&self) -> Span {
        match self {
            Self::Ident(_, span) => *span,
            Self::Index { span, .. } => *span,
            Self::CellIndex { span, .. } => *span,
            Self::Dot { span, .. } => *span,
            Self::MultiAssign(_, span) => *span,
        }
    }
}

/// Statement node in the AST.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Bare expression evaluation: `expr;` or `expr`
    Expr {
        expr: Expr,
        print_result: bool,
        span: Span,
    },
    /// Variable assignment: `target = value;`
    Assignment {
        target: AssignTarget,
        value: Expr,
        print_result: bool,
        span: Span,
    },
    /// Conditional branch: `if cond ... elseif ... else ... end`
    If {
        branches: Vec<(Expr, Vec<Stmt>)>,
        else_branch: Option<Vec<Stmt>>,
        span: Span,
    },
    /// Iteration loop: `for var = expr ... end`
    For {
        var: String,
        expr: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    /// Conditional loop: `while cond ... end`
    While {
        cond: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    /// Pattern match: `switch expr case ... otherwise ... end`
    Switch {
        expr: Expr,
        cases: Vec<(Vec<Expr>, Vec<Stmt>)>,
        otherwise: Option<Vec<Stmt>>,
        span: Span,
    },
    /// Error recovery: `try ... catch ME ... end`
    TryCatch {
        try_body: Vec<Stmt>,
        catch_var: Option<String>,
        catch_body: Vec<Stmt>,
        span: Span,
    },
    /// Loop break: `break`
    Break(Span),
    /// Loop continue: `continue`
    Continue(Span),
    /// Function return: `return`
    Return(Span),
    /// Global variable declaration: `global x y z`
    Global(Vec<String>, Span),
    /// Persistent variable declaration: `persistent a b`
    Persistent(Vec<String>, Span),
    /// Function definition: `function [out] = name(in) ... end`
    Function(FunctionDef),
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Self::Expr { span, .. } => *span,
            Self::Assignment { span, .. } => *span,
            Self::If { span, .. } => *span,
            Self::For { span, .. } => *span,
            Self::While { span, .. } => *span,
            Self::Switch { span, .. } => *span,
            Self::TryCatch { span, .. } => *span,
            Self::Break(span) => *span,
            Self::Continue(span) => *span,
            Self::Return(span) => *span,
            Self::Global(_, span) => *span,
            Self::Persistent(_, span) => *span,
            Self::Function(def) => def.span,
        }
    }
}

/// Definition of a user-defined MATLAB function.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub body: Vec<Stmt>,
    pub span: Span,
}
