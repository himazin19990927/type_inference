use pretty::{Doc, RcDoc};
use std::{collections::HashSet, convert::From, fmt};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinOp {
    Add,
    Mul,
    Gt,
    Lt,
    And,
    Or,
}

impl BinOp {
    pub fn precedence(&self) -> i8 {
        match self {
            BinOp::Add => 12,
            BinOp::Mul => 13,
            BinOp::Gt | BinOp::Lt => 7,
            BinOp::And | BinOp::Or => 6,
        }
    }

    pub fn fixity(&self) -> Fixity {
        match self {
            BinOp::Add | BinOp::Mul => Fixity::Left,
            BinOp::Gt | BinOp::Lt | BinOp::And | BinOp::Or => Fixity::None,
        }
    }

    pub fn to_doc(&self) -> RcDoc<()> {
        match self {
            BinOp::Add => RcDoc::text("+"),
            BinOp::Mul => RcDoc::text("*"),
            BinOp::Gt => RcDoc::text(">"),
            BinOp::Lt => RcDoc::text("<"),
            BinOp::And => RcDoc::text("&&"),
            BinOp::Or => RcDoc::text("||"),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Fixity {
    /// The operator is left-associative
    Left,
    /// The operator is right-associative
    Right,
    /// The operator is not-associative
    None,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Gt => write!(f, ">"),
            BinOp::Lt => write!(f, "<"),
            BinOp::And => write!(f, "&&"),
            BinOp::Or => write!(f, "||"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Num,
    Bool,
    Fun { arg: Box<Type>, ret: Box<Type> },

    TyVar(TyId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TyId(pub u32);

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Num => write!(f, "num"),
            Type::Bool => write!(f, "bool"),
            Type::Fun { arg, ret } => write!(f, "{}->{}", arg.as_ref(), ret.as_ref()),
            Type::TyVar(s) => write!(f, "a{}", s.0),
        }
    }
}

impl<'a> From<&'a AExpr> for &'a Type {
    fn from(item: &'a AExpr) -> Self {
        match item {
            AExpr::Num(_, ty) => ty,
            AExpr::Bool(_, ty) => ty,
            AExpr::Val(_, ty) => ty,
            AExpr::BinOp(_, _, _, ty) => ty,
            AExpr::Fun(_, _, ty) => ty,
            AExpr::App(_, _, ty) => ty,
        }
    }
}

impl From<AExpr> for Type {
    fn from(item: AExpr) -> Self {
        match item {
            AExpr::Num(_, ty) => ty,
            AExpr::Bool(_, ty) => ty,
            AExpr::Val(_, ty) => ty,
            AExpr::BinOp(_, _, _, ty) => ty,
            AExpr::Fun(_, _, ty) => ty,
            AExpr::App(_, _, ty) => ty,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Num(u32),
    Bool(bool),
    Val(String),
    BinOp(Box<Expr>, BinOp, Box<Expr>),
    Fun(String, Box<Expr>),
    App(Box<Expr>, Box<Expr>),
}

pub const PREC_CLOSURE: i8 = -40;
pub const PREC_JUMP: i8 = -30;
pub const PREC_RANGE: i8 = -10;
// The range 2..=14 is reserved for AssocOp binary operator precedences.
pub const PREC_PREFIX: i8 = 50;
pub const PREC_POSTFIX: i8 = 60;
pub const PREC_PAREN: i8 = 99;
pub const PREC_FORCE_PAREN: i8 = 100;

impl Expr {
    pub fn precedence(&self) -> i8 {
        match &self {
            Expr::Num(_) | Expr::Bool(_) | Expr::Val(_) | Expr::App(_, _) => PREC_PAREN,
            Expr::BinOp(_, op, _) => op.precedence(),
            Expr::Fun(_, _) => PREC_CLOSURE,
        }
    }

    pub fn to_doc(&self) -> RcDoc<()> {
        match self {
            Expr::Num(x) => RcDoc::as_string(x),
            Expr::Bool(x) => RcDoc::as_string(x),
            Expr::Val(val) => RcDoc::text(val),
            Expr::BinOp(lhs, op, rhs) => {
                let prec = op.precedence();
                let (prec_lhs, prec_rhs) = match op.fixity() {
                    Fixity::Left => (prec, prec + 1),
                    Fixity::Right => (prec + 1, prec),
                    Fixity::None => (prec + 1, prec + 1),
                };

                RcDoc::intersperse(
                    [
                        lhs.to_doc_maybe_paren(prec_lhs),
                        op.to_doc(),
                        rhs.to_doc_maybe_paren(prec_rhs),
                    ],
                    Doc::space(),
                )
            }
            Expr::Fun(arg, expr) => RcDoc::intersperse(
                [
                    RcDoc::text("fun"),
                    RcDoc::text(arg),
                    RcDoc::text("->"),
                    expr.to_doc(),
                ],
                Doc::space(),
            )
            .nest(1)
            .group(),

            Expr::App(fun, arg) => RcDoc::intersperse(
                [
                    fun.to_doc_maybe_paren(self.precedence()),
                    arg.to_doc_maybe_paren(self.precedence()),
                ],
                Doc::space(),
            ),
        }
    }

    fn to_doc_maybe_paren(&self, prec: i8) -> RcDoc<()> {
        self.to_doc_cond_paren(self.precedence() < prec)
    }

    fn to_doc_cond_paren(&self, needs_par: bool) -> RcDoc<()> {
        if needs_par {
            RcDoc::text("(")
                .append(self.to_doc())
                .append(RcDoc::text(")"))
        } else {
            self.to_doc()
        }
    }

    pub fn to_pretty(&self, width: usize) -> String {
        let mut w = Vec::new();
        self.to_doc().render(width, &mut w).unwrap();
        String::from_utf8(w).unwrap()
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Num(n) => write!(f, "{}", n),
            Expr::Bool(b) => write!(f, "{}", b),
            Expr::Val(id) => write!(f, "{}", id),
            Expr::BinOp(lhs, op, rhs) => write!(f, "({} {} {})", lhs.as_ref(), op, rhs.as_ref()),
            Expr::Fun(arg, expr) => write!(f, "fun {} -> {}", arg, expr.as_ref()),
            Expr::App(fun, arg) => write!(f, "(({}) {})", fun, arg),
        }
    }
}

pub fn collect_ids(ids: &mut HashSet<String>, expr: &Expr) {
    match expr {
        Expr::Val(id) => {
            ids.insert(id.clone());
        }
        Expr::BinOp(lhs, _, rhs) => {
            collect_ids(ids, lhs);
            collect_ids(ids, rhs);
        }
        Expr::Fun(id, expr) => {
            ids.insert(id.clone());
            collect_ids(ids, expr);
        }
        Expr::App(fun, arg) => {
            collect_ids(ids, fun);
            collect_ids(ids, arg);
        }
        _ => (),
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AExpr {
    Num(u32, Type),
    Bool(bool, Type),
    Val(String, Type),
    BinOp(Box<AExpr>, BinOp, Box<AExpr>, Type),
    Fun(String, Box<AExpr>, Type),
    App(Box<AExpr>, Box<AExpr>, Type),
}

impl fmt::Display for AExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AExpr::Num(n, t) => write!(f, "{}:{}", n, t),
            AExpr::Bool(b, t) => write!(f, "{}:{}", b, t),
            AExpr::Val(id, t) => write!(f, "{}:{}", id, t),
            AExpr::BinOp(lhs, op, rhs, t) => {
                write!(f, "({} {} {} ):{}", lhs.as_ref(), op, rhs.as_ref(), t)
            }
            AExpr::Fun(arg, expr, t) => match t {
                Type::Fun {
                    arg: argt,
                    ret: rest,
                } => {
                    write!(f, "(fun {}:{} -> {} ):{}", arg, argt, expr.as_ref(), rest)
                }
                _ => panic!("not a function"),
            },

            AExpr::App(fun, arg, t) => write!(f, "(({} ) {} ):{}", fun, arg, t),
        }
    }
}
