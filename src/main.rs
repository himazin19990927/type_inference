use std::env::args;

use type_inference::parser::expr;
use type_inference::{
    ast::Type,
    infer::{infer, Environment},
};

fn main() {
    let arg = args().nth(1).unwrap();
    let expr = expr::ExprParser::new().parse(arg.as_str()).unwrap();
    // println!("expr: {:?}", expr);

    let aexpr = infer(Environment::new(), expr);
    println!("result: {}", aexpr);
    println!("{}", Type::from(aexpr));
}