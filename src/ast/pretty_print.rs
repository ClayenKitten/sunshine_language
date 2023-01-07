use std::io::{Write, Result};

use crate::symbol_table::SymbolTable;

use super::{Visibility, item::{ItemKind, Module}};

const IDENT_WIDTH: usize = 4;
fn print_ident(w: &mut impl Write, ident: usize) -> Result<()> {
    write!(w, "{}", " ".repeat(ident*IDENT_WIDTH))
}

pub fn print_table(w: &mut impl Write, table: &SymbolTable) -> Result<()> {
    for (path, item) in table.declared.iter() {
        writeln!(w, "[{}]", path)?;
        if item.visibility == Visibility::Public {
            write!(w, "pub ")?;
        }
        match &item.kind {
            ItemKind::Module(
                Module::Inline(name)
                | Module::Loadable(name)
            ) => writeln!(w, "mod {};", name)?,
            ItemKind::Struct(s) => {
                writeln!(w, "struct {} {{", s.name)?;
                for field in s.fields.iter() {
                    writeln!(w, "    {}: {},", field.name, field.type_)?;
                }
                writeln!(w, "}}")?;
            },
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
                    writeln!(w, ") {{")?;
                }
                for stmt in func.body.statements.iter() {
                    writeln!(w, "{:#?};", stmt)?;
                }
                if let Some(expr) = &func.body.expression {
                    writeln!(w, "{:#?};", &expr)?;
                }
                writeln!(w, "}}")?;
            },
        }
        writeln!(w)?;
    }
    Ok(())
}
