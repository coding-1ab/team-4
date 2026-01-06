#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Create {
        table: Box<str>,
        columns: Vec<(Box<str>, Box<str>)>,
    },
    Insert {
        table: Box<str>,
        columns: Vec<Box<str>>,
        values: Vec<Expr>,
    },
    Select {
        table: Box<str>,
        columns: Vec<Box<str>>,
        filters: Option<Vec<Expr>>,
    },
    Update {
        table: Box<str>,
        assigns: Vec<(Expr, Expr)>,
        filters: Option<Vec<Expr>>,
    },
    Delete {
        table: Box<str>,
        filters: Option<Vec<Expr>>,
    },
    Drop {
        table: Box<str>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Null,
    Int(i64),
    Float(f64),
    Text(Box<str>),
    Ident(Box<str>),
    Unary {
        op: Box<str>,
        right: Box<Expr>,
    },
    Binary {
        op: Box<str>,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        name: Box<str>,
        args: Vec<Expr>,
    },
}
