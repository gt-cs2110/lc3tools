use std::collections::HashMap;

use lc3_ensemble::asm::{ObjectFile, SourceInfo, SymbolTable};
use lc3_ensemble::ast::asm::try_disassemble_line;

// Symbol access stuff
fn get_sym_source_from_obj(obj: &ObjectFile) -> Option<(&SymbolTable, &SourceInfo)> {
    let sym = obj.symbol_table()?;
    let src = sym.source_info()?;

    Some((sym, src))
}
fn add_mem_lines_from_obj(mem_lines: &mut HashMap<u16, String>, obj: &ObjectFile) {
    if let Some((sym, src_info)) = get_sym_source_from_obj(obj) {
        // For each source line in the object file,
        // if it maps to an address, add the mapping (addr, source line) to mem_lines.
        mem_lines.extend({
            sym.line_iter()
                .filter_map(|(lno, addr)| {
                    let span = src_info.line_span(lno)?;
                    let text = src_info.source().get(span)?.to_string();
                    Some((addr, text))
                })
        });

        // Update sources to better handle .stringz:
        let labels = obj.addr_iter()
            .filter_map(|(addr, m_val)| match m_val {
                Some(val @ 0x0020..0x007F) => Some((addr, char::from(val as u8).to_string())),
                _ => None
            });

        for (addr, label) in labels {
            let new_label = match mem_lines.get(&addr) {
                Some(orig_label) => format!("{orig_label} ({label})"),
                None => label,
            };
            mem_lines.insert(addr, new_label);
        }
    }
}
//

#[derive(Default)]
pub(crate) struct ObjContents {
    obj_file: Option<ObjectFile>,
    mem_lines: HashMap<u16, String>,
}
impl ObjContents {
    pub(crate) fn get_mem_line(&self, addr: u16) -> &str {
        self.mem_lines.get(&addr).map_or("", |s| s)
    }
    pub(crate) fn set_mem_line(&mut self, addr: u16, value: u16) {
        let string = if (0x0020..0x007F).contains(&value) {
            // ASCII
            char::from(value as u8).to_string()
        } else {
            // Disassemble logic
            match try_disassemble_line(value) {
                Some(s) => format!("*{s}"),
                None => String::new(),
            }
        };
    
        self.mem_lines.insert(addr, string);
    }

    pub(crate) fn load_contents(&mut self, obj: ObjectFile) {
        // Set mem lines:
        add_mem_lines_from_obj(&mut self.mem_lines, lc3_ensemble::sim::_os_obj_file());
        add_mem_lines_from_obj(&mut self.mem_lines, &obj);
        //

        self.obj_file.replace(obj);
    }

    pub(crate) fn clear(&mut self) {
        self.obj_file.take();
        self.mem_lines.clear();
    }
    
    pub(crate) fn get_sym_source(&self) -> Option<(&SymbolTable, &SourceInfo)> {
        get_sym_source_from_obj(self.obj_file.as_ref()?)
    }
}