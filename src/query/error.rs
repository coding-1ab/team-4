pub type Result<T> = std::result::Result<T, QueryErr>;

#[derive(Debug)]
pub enum QueryErr {
    ReservedKeyword,
    UnexpectedEof,
    InvalidNum(String),
    UnterminatedText,
    InvalidIdent,
    InvalidToken(char),
    UnexpectedToken { expected: String, found: String },
    InvalidExpr(String),
}
