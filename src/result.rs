use pretty::{Doc, RcDoc};

use crate::ast::{AExpr, Expr, TyId, Type};

pub struct InferResult {
    pub input: Expr,
    pub annotated: AExpr,
    pub constraints: Vec<(Type, Type)>,
    pub process: InferProcess,
    pub output: AExpr,
}

pub struct InferProcess {
    pub operation: InferOperation,
    pub process: Vec<InferProcess>,
}

impl InferProcess {
    pub fn to_doc(&self) -> RcDoc<()> {
        if self.process.is_empty() {
            self.operation
                .to_doc_name()
                .append(Doc::space())
                .append(self.operation.to_doc_operation())
        } else {
            let mut lines = Vec::new();
            lines.push(RcDoc::text("{"));
            lines.extend(self.process.iter().map(|process| process.to_doc()));
            self.operation.to_doc_name().append(RcDoc::space()).append(
                RcDoc::intersperse(lines, Doc::hardline())
                    .nest(4)
                    .append(RcDoc::line())
                    .append("}"),
            )
        }
    }
}

pub enum InferOperation {
    Unify {
        constraints: Vec<(Type, Type)>,
        result: Vec<(TyId, Type)>,
    },
    UnifyOne {
        ty1: Type,
        ty2: Type,
        result: Vec<(TyId, Type)>,
    },
    Apply {
        substitudes: Vec<(TyId, Type)>,
        ty: Type,
        result: Type,
    },
    Substitude {
        u: Type,
        x: TyId,
        t: Type,
        result: Type,
    },
}

impl InferOperation {
    pub fn to_doc_name(&self) -> RcDoc<()> {
        match &self {
            InferOperation::Unify { .. } => RcDoc::text("Unify"),
            InferOperation::UnifyOne { .. } => RcDoc::text("UnifyOne"),
            InferOperation::Apply { .. } => RcDoc::text("Apply"),
            InferOperation::Substitude { .. } => RcDoc::text("Substitude"),
        }
    }

    pub fn to_doc_operation(&self) -> RcDoc<()> {
        match &self {
            InferOperation::Unify {
                constraints,
                result,
            } => RcDoc::intersperse(
                [
                    constraints_to_doc(constraints),
                    RcDoc::text("=>"),
                    substitudes_to_doc(result),
                ],
                Doc::space(),
            ),
            InferOperation::UnifyOne { ty1, ty2, result } => RcDoc::intersperse(
                [
                    ty1.to_doc(),
                    RcDoc::text("<->"),
                    ty2.to_doc(),
                    RcDoc::text("=>"),
                    substitudes_to_doc(result),
                ],
                Doc::space(),
            ),
            InferOperation::Apply {
                substitudes,
                ty,
                result,
            } => RcDoc::intersperse(
                [
                    ty.to_doc(),
                    RcDoc::text("->"),
                    substitudes_to_doc(substitudes),
                    RcDoc::text("=>"),
                    result.to_doc(),
                ],
                Doc::space(),
            ),
            InferOperation::Substitude { u, x, t, result } => RcDoc::intersperse(
                [
                    RcDoc::text("(")
                        .append(RcDoc::intersperse(
                            [u.to_doc(), x.to_doc(), t.to_doc()],
                            RcDoc::text(",").append(Doc::space()),
                        ))
                        .append(")"),
                    RcDoc::text("=>"),
                    result.to_doc(),
                ],
                Doc::space(),
            ),
        }
    }
}

pub struct UnifyingStep {
    pub constraint: (Type, Type),
    pub substitudes: Vec<(TyId, Type)>,
}

impl UnifyingStep {
    pub fn to_doc(&self) -> RcDoc<()> {
        RcDoc::text("(")
            .append(constraint_to_doc(&self.constraint))
            .append(")")
            .append(Doc::hardline())
            .append(substitudes_to_doc(&self.substitudes))
    }
}

impl InferResult {
    pub fn to_doc(&self) -> RcDoc<()> {
        let offset = 4;
        RcDoc::intersperse(
            [
                RcDoc::text("input:")
                    .append(RcDoc::hardline().append(self.input.to_doc()).nest(offset)),
                RcDoc::text("annotated:").append(
                    RcDoc::hardline()
                        .append(self.annotated.to_doc())
                        .nest(offset),
                ),
                RcDoc::text("constraints:").append(
                    RcDoc::hardline()
                        .append(constraints_to_doc(&self.constraints))
                        .nest(offset),
                ),
                RcDoc::text("process")
                    .append(RcDoc::hardline().append(self.process.to_doc()).nest(offset)),
                RcDoc::text("output:")
                    .append(RcDoc::hardline().append(self.output.to_doc()).nest(offset)),
            ],
            Doc::line(),
        )
    }

    pub fn to_pretty(&self, width: usize) -> String {
        let mut w = Vec::new();
        self.to_doc().render(width, &mut w).unwrap();
        String::from_utf8(w).unwrap()
    }
}

fn constraints_to_doc(constraints: &Vec<(Type, Type)>) -> RcDoc<()> {
    RcDoc::text("[")
        .append(RcDoc::intersperse(
            constraints
                .iter()
                .map(|constraint| constraint_to_doc(constraint)),
            RcDoc::text(",").append(Doc::space()),
        ))
        .append("]")
}

fn constraint_to_doc(constraint: &(Type, Type)) -> RcDoc<()> {
    let (lhs, rhs) = constraint;
    RcDoc::intersperse([lhs.to_doc(), RcDoc::text("="), rhs.to_doc()], Doc::space())
}

fn substitudes_to_doc(subs: &Vec<(TyId, Type)>) -> RcDoc<()> {
    RcDoc::text("[")
        .append(RcDoc::intersperse(
            subs.iter().map(|sub| substitute_to_doc(sub)),
            RcDoc::text(",").append(Doc::space()),
        ))
        .append("]")
}

fn substitute_to_doc(sub: &(TyId, Type)) -> RcDoc<()> {
    let (id, ty) = sub;
    RcDoc::text("(")
        .append(RcDoc::intersperse(
            [RcDoc::text("a").append(RcDoc::as_string(id.0)), ty.to_doc()],
            RcDoc::text(",").append(Doc::space()),
        ))
        .append(")")
}
