use std::{
    fmt::Display,
    io::{Result, Write},
};

use crate::{item_table::ItemTable, path::AbsolutePath};

use super::{
    expression::{Block, Expression, Literal},
    item::{Item, ItemKind, Module, Visibility},
    statement::{LetStatement, Statement},
};

pub fn print_table(w: impl Write + 'static, table: &ItemTable) -> Result<()> {
    let mut printer = Printer {
        writer: Box::new(w),
        indent: 0,
    };
    for (path, item) in table.declared.iter() {
        printer.print_item(path, item)?;
    }
    Ok(())
}

struct Printer {
    writer: Box<dyn Write>,
    indent: usize,
}

impl Printer {
    /// Width of a single indentation.
    const IDENT_WIDTH: usize = 4;

    fn print_item(&mut self, path: &AbsolutePath, item: &Item) -> Result<()> {
        self.println(format!("[{path}]"))?;
        if let Visibility::Public = item.visibility {
            write!(self.writer, "PUB ")?
        }
        let span = format!("@ {}/{}", item.span.start, item.span.end);
        match &item.kind {
            ItemKind::Module(Module::Inline(name) | Module::Loadable(name)) => {
                writeln!(self.writer, "MOD {name}; {span}")?
            }
            ItemKind::Struct(s) => {
                self.println(format!("STRUCT {} {span}", s.name))?;
                self.with_indent(|printer| {
                    for field in s.fields.iter() {
                        printer.println(format!("{}: {}", field.name, field.type_,))?;
                    }
                    Ok(())
                })?;
            }
            ItemKind::Function(func) => {
                self.println(format!("FN `{}` {span}", func.name))?;
                self.with_indent(|printer| {
                    if !func.params.is_empty() {
                        printer.println("PARAMS")?;
                        printer.with_indent(|printer| {
                            for param in func.params.iter() {
                                printer.println(format!("`{}`: `{}`", param.name, param.type_))?;
                            }
                            Ok(())
                        })?;
                    }
                    if let Some(ret_type) = &func.return_type {
                        printer.println(format!("RETURN `{ret_type}`"))?;
                    }
                    printer.println("BODY")?;
                    printer.print_block(&func.body)?;
                    Ok(())
                })?;
            }
        }
        self.newline()?;
        Ok(())
    }

    fn print_stmt(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::ExprStmt(expr) => {
                self.print_expr(expr)?;
            }
            Statement::LetStmt(LetStatement { name, type_, value }) => {
                if let Some(type_) = type_ {
                    self.println(format!("LET `{name}`: `{type_}`"))?;
                } else {
                    self.println(format!("LET `{name}`"))?;
                }
                if let Some(value) = value {
                    self.with_indent(|printer| printer.print_expr(value))?;
                }
            }
            Statement::Assignment {
                assignee,
                operator,
                expression,
            } => {
                self.println("ASSIGN")?;
                self.with_indent(|printer| {
                    printer.println(format!("ASSIGNEE `{assignee}`"))?;
                    printer.println(format!("OPERATOR `{operator}`"))?;
                    Ok(())
                })?;
                self.println("VALUE".to_string())?;
                self.with_indent(|printer| printer.print_expr(expression))?;
            }
            Statement::Return(expr) => {
                self.println("RETURN")?;
                self.with_indent(|printer| printer.print_expr(expr))?;
            }
            Statement::Break => self.println("BREAK")?,
        }
        Ok(())
    }

    fn print_expr(&mut self, expr: &Expression) -> Result<()> {
        match expr {
            Expression::Block(block) => self.print_block(block)?,
            Expression::If {
                condition,
                body,
                else_body,
            } => {
                self.println("IF")?;
                self.with_indent(|printer| printer.print_expr(condition))?;

                self.println("THEN")?;
                self.print_block(body)?;

                if let Some(else_body) = else_body {
                    self.println("ELSE")?;
                    self.print_block(else_body)?;
                }
            }
            Expression::While { condition, body } => {
                self.println("WHILE")?;
                self.with_indent(|printer| printer.print_expr(condition))?;
                self.println("BODY")?;
                self.print_block(body)?;
            }
            Expression::For { var, expr, body } => {
                self.println(format!("FOR `{var}`"))?;
                self.println("IN")?;
                self.with_indent(|printer| printer.print_expr(expr))?;
                self.println("BODY")?;
                self.print_block(body)?;
            }
            Expression::Literal(Literal::Number(num)) => self.println(format!("`{num}`"))?,
            Expression::Literal(Literal::String(s)) => self.println(format!("`\"{s}\"`"))?,
            Expression::Literal(Literal::Boolean(true)) => self.println("`true`")?,
            Expression::Literal(Literal::Boolean(false)) => self.println("`false`")?,
            Expression::Var(var) => self.println(var)?,
            Expression::Unary { op, value } => {
                self.println(format!("UNARY `{op}`"))?;
                self.with_indent(|printer| printer.print_expr(value))?;
            }
            Expression::Binary { op, left, right } => {
                self.println(format!("BINARY `{op}`"))?;
                self.with_indent(|printer| {
                    printer.println("LEFT")?;
                    printer.with_indent(|printer| printer.print_expr(left))?;
                    printer.println("RIGHT")?;
                    printer.with_indent(|printer| printer.print_expr(right))?;
                    Ok(())
                })?;
            }
            Expression::FnCall { path, params } => {
                self.println(format!("FNCALL `{path}`"))?;
                self.with_indent(|printer| {
                    for param in params {
                        printer.print_expr(param)?;
                    }
                    Ok(())
                })?;
            }
        }
        Ok(())
    }

    fn print_block(&mut self, block: &Block) -> Result<()> {
        self.with_indent(|printer| {
            for stmt in block.statements.iter() {
                printer.print_stmt(stmt)?;
            }
            if let Some(expr) = &block.expression {
                printer.print_expr(expr)?;
            }
            Ok(())
        })
    }

    fn with_indent(&mut self, f: impl Fn(&mut Self) -> Result<()>) -> Result<()> {
        self.indent += 1;
        f(self)?;
        self.indent -= 1;
        Ok(())
    }

    fn newline(&mut self) -> Result<()> {
        writeln!(self.writer)
    }

    fn println(&mut self, line: impl Display) -> Result<()> {
        self.print_indent()?;
        write!(self.writer, "{}", line)?;
        self.newline()?;
        Ok(())
    }

    fn print_indent(&mut self) -> Result<()> {
        write!(
            self.writer,
            "{}",
            " ".repeat(self.indent * Self::IDENT_WIDTH)
        )
    }
}
