use matlab_diagnostics::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Assignment {
        target: AssignTarget,
        value: Expr,
        suppress_output: bool,
        span: Span,
    },
    Expr {
        expr: Expr,
        suppress_output: bool,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignTarget {
    Ident(String, Span),
    Index {
        name: String,
        indices: Vec<Expr>,
        span: Span,
    },
    /// `[X, Y] = f(...)` multi-output assignment (each name pro-supplied).
    Multi(Vec<(String, Span)>, Span),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64, Span),
    StringLit(String, Span),
    Variable(String, Span),
    /// `a:step:b` / `a:b` (step defaults to 1). Evaluates to a row vector.
    Range {
        start: Box<Expr>,
        step: Option<Box<Expr>>,
        stop: Box<Expr>,
        span: Span,
    },
    /// The reserved `end` keyword inside indexing.
    End(Span),
    /// A bare `:` inside indexing (all elements along that dimension).
    ColonAll(Span),
    MatrixLit(Vec<Vec<Expr>>, Span),
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },
    Call {
        callee: String,
        args: Vec<Expr>,
        span: Span,
    },
    Transpose {
        expr: Box<Expr>,
        conjugate: bool,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Number(_, s)
            | Expr::StringLit(_, s)
            | Expr::Variable(_, s)
            | Expr::MatrixLit(_, s)
            | Expr::End(s)
            | Expr::ColonAll(s)
            | Expr::Binary { span: s, .. }
            | Expr::Unary { span: s, .. }
            | Expr::Call { span: s, .. }
            | Expr::Transpose { span: s, .. } => *s,
            Expr::Range { start, stop, .. } => start.span().merge(stop.span()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,    // +
    Sub,    // -
    Mul,    // * (Matrix multiply)
    Div,    // / (Matrix right division)
    Pow,    // ^ (Matrix power)
    DotMul, // .* (Element-wise multiply)
    DotDiv, // ./ (Element-wise divide)
    DotPow, // .^ (Element-wise power)
    Eq,     // ==
    Ne,     // ~=
    Lt,     // <
    Le,     // <=
    Gt,     // >
    Ge,     // >=
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Pos, // +
    Neg, // -
}
