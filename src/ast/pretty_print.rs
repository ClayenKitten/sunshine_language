use std::io::{Result, Write};

use crate::{
    item_table::ItemTable,
    lexer::number::{Base, Number},
};

use super::{
    block::Block,
    expression::{Expression, For, FunctionCall, If, Literal, While},
    item::{ItemKind, Module},
    statement::{Assignment, LetStatement, Statement},
    Visibility,
};

pub fn print_table(w: &mut impl Write, table: &ItemTable) -> Result<()> {
    for (path, item) in table.declared.iter() {
        writeln!(w, "[{}]", path)?;
        if item.visibility == Visibility::Public {
            write!(w, "pub ")?;
        }
        match &item.kind {
            ItemKind::Module(Module::Inline(name) | Module::Loadable(name)) => {
                writeln!(w, "mod {};", name)?
            }
            ItemKind::Struct(s) => {
                writeln!(w, "struct {} {{", s.name)?;
                for field in s.fields.iter() {
                    writeln!(w, "    {}: {},", field.name, field.type_)?;
                }
                writeln!(w, "}}")?;
            }
            ItemKind::Function(func) => {
                write!(w, "fn {} (", func.name)?;
                for param in func.params.iter() {
                    write!(w, "\n    {}: {},", param.name, param.type_)?;
                }
                if !func.params.is_empty() {
                    writeln!(w)?;
                }
                if let Some(ret_type) = &func.return_type {
                    writeln!(w, ") -> {} {{", ret_type)?;
                } else {
                    write!(w, ") ")?;
                }
                print_block(w, &func.body, 0)?;
            }
        }
        writeln!(w)?;
    }
    Ok(())
}

fn print_stmt(w: &mut impl Write, stmt: &Statement, ident: usize) -> Result<()> {
    print_ident(w, ident)?;
    match stmt {
        Statement::ExprStmt(expr) => {
            print_expr(w, &expr, ident)?;
            writeln!(w, ";")?;
        }
        Statement::LetStmt(LetStatement { name, type_, value }) => {
            write!(w, "let {name}")?;
            if let Some(type_) = type_ {
                write!(w, ": {type_}")?;
            }
            if let Some(value) = value {
                write!(w, " = ")?;
                print_expr(w, value, ident)?;
            }
            writeln!(w, ";")?;
        }
        Statement::Assignment(Assignment {
            assignee,
            operator,
            value,
        }) => {
            write!(w, "{} {} ", assignee, operator.0)?;
            print_expr(w, value, ident)?;
            writeln!(w, ";")?;
        }
        Statement::Return(expr) => {
            write!(w, "return ")?;
            print_expr(w, expr, ident)?;
            writeln!(w, ";")?;
        }
        Statement::Break => writeln!(w, "break;")?,
    }
    Ok(())
}

fn print_expr(w: &mut impl Write, expr: &Expression, ident: usize) -> Result<()> {
    match expr {
        Expression::Block(block) => print_block(w, block, ident)?,
        Expression::If(If {
            condition,
            body,
            else_body,
        }) => {
            write!(w, "if ")?;
            print_expr(w, &condition, ident)?;
            write!(w, " ")?;
            print_block(w, body, ident)?;
            if let Some(else_body) = else_body {
                write!(w, " else ")?;
                print_block(w, else_body, ident)?;
            }
        }
        Expression::While(While { condition, body }) => {
            write!(w, "while ")?;
            print_expr(w, &condition, ident)?;
            print_block(w, body, ident)?;
        }
        Expression::For(For { var, expr, body }) => {
            write!(w, "for {var} in ")?;
            print_expr(w, expr, ident)?;
            print_block(w, body, ident)?;
        }
        Expression::Ident(ident) => write!(w, "{}", ident)?,
        Expression::Literal(Literal::Number(Number {
            integer,
            fraction,
            base,
        })) => {
            match base {
                Base::Binary => write!(w, "0b")?,
                Base::Octal => write!(w, "0o")?,
                Base::Hexadecimal => write!(w, "0x")?,
                Base::Decimal => {}
            }
            write!(w, "{integer}")?;
            if let Some(fraction) = fraction {
                write!(w, "{fraction}")?;
            }
        }
        Expression::Literal(Literal::String(s)) => write!(w, "\"{}\"", s)?,
        Expression::Literal(Literal::Boolean(true)) => write!(w, "true")?,
        Expression::Literal(Literal::Boolean(false)) => write!(w, "false")?,
        Expression::Unary { op, value } => {
            write!(w, "{}", op.0)?;
            print_expr(w, &value, ident)?;
        }
        Expression::Binary { op, left, right } => {
            write!(w, "(")?;
            print_expr(w, &left, ident)?;
            write!(w, " {} ", op.0)?;
            print_expr(w, &right, ident)?;
            write!(w, ")")?;
        }
        Expression::FnCall(FunctionCall { name, params }) => {
            write!(w, "{name}(")?;
            for param in params {
                print_expr(w, &param, ident)?;
                write!(w, ",")?;
            }
            write!(w, ")")?;
        }
        Expression::Var(var) => write!(w, "{var}")?,
    }
    Ok(())
}

fn print_block(w: &mut impl Write, block: &Block, ident: usize) -> Result<()> {
    writeln!(w, "{{")?;
    for stmt in block.statements.iter() {
        print_stmt(w, &stmt, ident + 1)?;
    }
    if let Some(expr) = &block.expression {
        print_ident(w, ident + 1)?;
        print_expr(w, expr, ident + 1)?;
        writeln!(w)?;
    }
    print_ident(w, ident)?;
    write!(w, "}}")?;
    Ok(())
}

const IDENT_WIDTH: usize = 4;
fn print_ident(w: &mut impl Write, ident: usize) -> Result<()> {
    write!(w, "{}", " ".repeat(ident * IDENT_WIDTH))
}
