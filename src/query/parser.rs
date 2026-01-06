use super::error::{QueryErr, Result};
use super::lexer::{Lexer, Token};
use std::mem::{discriminant, replace};

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

impl Block {
    pub fn new(stmts: Vec<Stmt>) -> Self {
        Self { stmts }
    }

    pub fn single(stmt: Stmt) -> Self {
        Self { stmts: vec![stmt] }
    }
}

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
        filters: Option<Box<Expr>>,
    },
    Update {
        table: Box<str>,
        assigns: Vec<(Box<str>, Expr)>,
        filters: Option<Box<Expr>>,
    },
    Delete {
        table: Box<str>,
        filters: Option<Box<Expr>>,
    },
    Drop {
        table: Box<str>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Null,
    Bool(bool),
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
}

impl Expr {
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

pub struct Parser {
    lexer: Lexer,
    curr: Token,
    peek: Token,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Result<Self> {
        let curr = lexer.next()?;
        let peek = lexer.next()?;
        Ok(Self { lexer, curr, peek })
    }

    fn next(&mut self) -> Result<Token> {
        Ok(replace(
            &mut self.curr,
            replace(&mut self.peek, self.lexer.next()?),
        ))
    }

    fn expect(&mut self, token: &Token) -> Result<()> {
        if discriminant(&self.curr) == discriminant(token) {
            self.next()?;
            Ok(())
        } else {
            Err(QueryErr::UnexpectedToken {
                expected: format!("{:?}", token),
                found: format!("{:?}", self.curr),
            })
        }
    }

    fn maybe(&mut self, token: &Token) -> Result<bool> {
        if discriminant(&self.curr) == discriminant(token) {
            self.next()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn parse(&mut self) -> Result<Block> {
        self.parse_block(&[Token::Eof])
    }

    fn parse_block(&mut self, terms: &[Token]) -> Result<Block> {
        let mut stmts = Vec::new();
        while !terms
            .iter()
            .any(|t| discriminant(t) == discriminant(&self.curr))
        {
            if self.curr == Token::Semicolon {
                self.next()?;
                continue;
            }
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
        }
        Ok(Block::new(stmts))
    }

    fn consume_ident(&mut self) -> Result<Box<str>> {
        match self.next()? {
            Token::Ident(name) => Ok(name.into_boxed_str()),
            tok => Err(QueryErr::UnexpectedToken {
                expected: "<ident>".into(),
                found: format!("{:?}", tok),
            }),
        }
    }

    pub fn parse_stmt(&mut self) -> Result<Stmt> {
        match &self.curr {
            Token::Create => self.parse_create(),
            Token::Insert => self.parse_insert(),
            Token::Select => self.parse_select(),
            Token::Update => self.parse_update(),
            Token::Delete => self.parse_delete(),
            Token::Drop => self.parse_drop(),
            tok => Err(QueryErr::UnexpectedToken {
                expected: "<stmt>".into(),
                found: format!("{:?}", tok),
            }),
        }
    }

    fn parse_create(&mut self) -> Result<Stmt> {
        // CREATE TABLE <table> (<col1> <type>, <col2> <type>, ...)
        // CREATE TABLE <table> 파싱
        self.expect(&Token::Create)?;
        self.expect(&Token::Table)?;
        let table = self.consume_ident()?;
        // (<col1> <type>, <col2> <type>, ...) 파싱
        self.expect(&Token::LParen)?;
        let mut columns = Vec::new();
        loop {
            let col_name = self.consume_ident()?;
            let col_type = self.consume_ident()?;
            columns.push((col_name, col_type));
            match self.next()? {
                Token::Comma => continue,
                Token::RParen => break,
                tok => {
                    return Err(QueryErr::UnexpectedToken {
                        expected: "',' or ')'".into(),
                        found: format!("{:?}", tok),
                    });
                }
            }
        }
        Ok(Stmt::Create { table, columns })
    }

    fn parse_insert(&mut self) -> Result<Stmt> {
        // INSERT INTO <table> [(<col1>, <col2>, ...)] VALUES (<val1>, <val2>, ...)
        // INSERT INTO <table> 파싱
        self.expect(&Token::Insert)?;
        self.expect(&Token::Into)?;
        let table = self.consume_ident()?;
        // [(<col1>, <col2>, ...)] 파싱
        let mut columns = Vec::new();
        if self.maybe(&Token::LParen)? {
            // 괄호가 있는 경우, 부분 칼럼 파싱
            loop {
                columns.push(self.consume_ident()?);
                match self.next()? {
                    Token::Comma => continue,
                    Token::RParen => break,
                    tok => {
                        return Err(QueryErr::UnexpectedToken {
                            expected: "',' or ')'".into(),
                            found: format!("{:?}", tok),
                        });
                    }
                }
            }
        }
        // VALUES (<val1>, <val2>, ...) 파싱
        self.expect(&Token::Values)?;
        self.expect(&Token::LParen)?;
        let mut values = Vec::new();
        loop {
            values.push(self.parse_expr()?);
            match self.next()? {
                Token::Comma => continue,
                Token::RParen => break,
                tok => {
                    return Err(QueryErr::UnexpectedToken {
                        expected: "',' or ')'".into(),
                        found: format!("{:?}", tok),
                    });
                }
            }
        }
        Ok(Stmt::Insert {
            table,
            columns,
            values,
        })
    }

    fn parse_select(&mut self) -> Result<Stmt> {
        // SELECT <col1>, <col2>, ... FROM <table> [WHERE ...]
        // SELECT <col1>, <col2>, ... 파싱
        self.expect(&Token::Select)?;
        let mut columns = Vec::new();
        if &self.curr == &Token::Mul {
            // 전체 선택 `*` 처리
            self.next()?;
            columns.push("*".into());
        } else {
            // 괄호 선택 처리
            let paren = self.maybe(&Token::LParen)?;
            loop {
                columns.push(self.consume_ident()?);
                if !self.maybe(&Token::Comma)? {
                    break;
                }
            }
            // 만약 괄호가 있다면, 반드시 닫기
            if paren {
                self.expect(&Token::RParen)?;
            }
        }
        // FROM <table> 파싱
        self.expect(&Token::From)?;
        let table = self.consume_ident()?;
        // TODO: [WHERE ...] 파싱
        let filters = None;
        Ok(Stmt::Select {
            table,
            columns,
            filters,
        })
    }

    fn parse_update(&mut self) -> Result<Stmt> {
        // UPDATE <table> SET <col1> = <val1>, <col2> = <val2>, ... [WHERE ...]
        // UPDATE <table> SET 파싱
        self.expect(&Token::Update)?;
        let table = self.consume_ident()?;
        self.expect(&Token::Set)?;
        // <col1> = <val1>, <col2> = <val2>, ... 파싱
        let mut assigns = Vec::new();
        loop {
            let col = self.consume_ident()?;
            self.expect(&Token::Eq)?;
            let val = self.parse_expr()?;
            assigns.push((col, val));
            if !self.maybe(&Token::Comma)? {
                break;
            }
        }
        // TODO: [WHERE ...] 파싱
        let filters = None;
        Ok(Stmt::Update {
            table,
            assigns,
            filters,
        })
    }

    fn parse_delete(&mut self) -> Result<Stmt> {
        // DELETE FROM <table> [WHERE ...]
        // DELETE FROM <table> 파싱
        self.expect(&Token::Delete)?;
        self.expect(&Token::From)?;
        let table = self.consume_ident()?;
        // TODO: [WHERE ...] 파싱
        let filters = None;
        Ok(Stmt::Delete { table, filters })
    }

    fn parse_drop(&mut self) -> Result<Stmt> {
        // DROP TABLE <table>
        self.expect(&Token::Drop)?;
        self.expect(&Token::Table)?;
        let table = self.consume_ident()?;
        Ok(Stmt::Drop { table })
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        match self.next()? {
            Token::Null => Ok(Expr::Null),
            Token::Bool(b) => Ok(Expr::Bool(b)),
            Token::Num(n) => {
                if let Ok(i) = n.parse::<i64>() {
                    Ok(Expr::Int(i))
                } else if let Ok(f) = n.parse::<f64>() {
                    Ok(Expr::Float(f))
                } else {
                    Err(QueryErr::InvalidExpr(format!(
                        "Invalid number: {}",
                        n.to_string() + ""
                    )))
                }
            }
            Token::Text(t) => Ok(Expr::Text(t.into_boxed_str())),
            Token::Ident(i) => Ok(Expr::Ident(i.into_boxed_str())),
            tok => Err(QueryErr::UnexpectedToken {
                expected: "<expr>".into(),
                found: format!("{:?}", tok),
            }),
            // TODO: Unary, Binary, Call 파싱
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::lexer::Lexer;

    #[test]
    fn test_parse_drop_table() {
        let lexer = Lexer::new("DROP TABLE users");
        let mut parser = Parser::new(lexer).unwrap();
        let stmt = parser.parse_stmt().unwrap();
        assert_eq!(
            stmt,
            Stmt::Drop {
                table: "users".into()
            }
        );
    }

    #[test]
    fn test_parse_select_star() {
        let lexer = Lexer::new("SELECT * FROM users");
        let mut parser = Parser::new(lexer).unwrap();
        let stmt = parser.parse_stmt().unwrap();
        assert_eq!(
            stmt,
            Stmt::Select {
                table: "users".into(),
                columns: vec!["*".into()],
                filters: None,
            }
        );
    }

    #[test]
    fn test_parse_select_cols() {
        let lexer = Lexer::new("SELECT id, name FROM users");
        let mut parser = Parser::new(lexer).unwrap();
        let stmt = parser.parse_stmt().unwrap();
        assert_eq!(
            stmt,
            Stmt::Select {
                table: "users".into(),
                columns: vec!["id".into(), "name".into()],
                filters: None,
            }
        );
    }

    #[test]
    fn test_parse_insert() {
        let lexer = Lexer::new("INSERT INTO users (id, name) VALUES (1, 'Alice')");
        let mut parser = Parser::new(lexer).unwrap();
        let stmt = parser.parse_stmt().unwrap();
        assert_eq!(
            stmt,
            Stmt::Insert {
                table: "users".into(),
                columns: vec!["id".into(), "name".into()],
                values: vec![Expr::Int(1), Expr::Text("Alice".into())],
            }
        );
    }
}
