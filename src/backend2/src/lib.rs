mod print;

use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

use lc3_ensemble::asm::{assemble_debug, ObjectFile};
use lc3_ensemble::parse::parse_ast;
use lc3_ensemble::sim::Simulator;
use neon::prelude::*;
use once_cell::sync::Lazy;
use print::{report_error, report_simple, PrintBuffer};

static PRINT_BUFFER: Mutex<PrintBuffer> = Mutex::new(PrintBuffer::new());
static SIMULATOR_CONTENTS: Lazy<Mutex<SimState>> = Lazy::new(|| {
    Mutex::new(SimState {
        sim: Simulator::new(),
        obj: None,
    })
});
struct SimState {
    sim: Simulator,
    obj: Option<ObjectFile>
}

//--------- CONFIG FUNCTIONS ---------//

fn init(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn() -> Result<()>
    // TODO: Determine whether ensemble requires an init.
    Ok(cx.undefined())
}
fn set_enable_liberal_asm(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (enable: bool) -> Result<()>
    // TODO: What does liberal ASM do?
    Ok(cx.undefined())
}
fn set_ignore_privilege(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn(enable: bool) -> Result<()>
    // TODO: Implement ignore privilege
    Ok(cx.undefined())
}

//--------- CONSOLE FUNCTIONS ---------//

fn get_and_clear_output(mut cx: FunctionContext) -> JsResult<JsString> {
    // fn() -> Result<String>
    let string = PRINT_BUFFER.lock().unwrap().take();
    Ok(cx.string(string))
}

fn clear_output(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn() -> Result<()>
    PRINT_BUFFER.lock().unwrap().take();
    Ok(cx.undefined())
}

//--------- EDITOR/ASSEMBLER FUNCTIONS ---------//

fn convert_bin(mut _cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn(fp: String) -> Result<()>

    // .bin files are files that have ASCII binary instead of assembly code.
    // Maybe will be implemented later? idk.
    unimplemented!("ConvertBin");
}

fn assemble(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fp: String) -> Result<()>
    let fp = cx.argument::<JsString>(0)?.value(&mut cx);
    let in_path = AsRef::<Path>::as_ref(&fp);
    let out_path = in_path.with_extension("obj");
    
    // should be unreachable cause frontend validates IO
    let src = std::fs::read_to_string(in_path).unwrap();

    let ast = parse_ast(&src)
        .map_err(|e| report_error(e, in_path, &src, &mut cx, &mut PRINT_BUFFER.lock().unwrap()))?;
    let asm = assemble_debug(ast, &src)
        .map_err(|e| report_error(e, in_path, &src, &mut cx, &mut PRINT_BUFFER.lock().unwrap()))?;
    
    std::fs::write(&out_path, asm.write_bytes())
        .map_err(|e| report_simple(in_path, e, &mut cx, &mut PRINT_BUFFER.lock().unwrap()))?;

    writeln!(PRINT_BUFFER.lock().unwrap(), "Successfully assembled {} into {}", in_path.display(), out_path.display()).unwrap();
    Ok(cx.undefined())
}

//--------- SIMULATOR FUNCTIONS ---------//

fn get_curr_sym_table(mut cx: FunctionContext) -> JsResult<JsObject> {
    // fn (fp: String) -> Result<Object>
    todo!()
}
fn load_object_file(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fp: string) -> Result<()>
    todo!()
}
fn restart_machine(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn () -> Result<()>
    todo!()
}
fn reinitialize_machine(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn () -> Result<()>
    todo!()
}
fn randomize_machine(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fn(err) -> ()) -> Result<()>
    todo!()
}
fn run(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fn(err) -> ()) -> Result<()>
    todo!()
}
fn run_until_halt(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fn(err) -> ()) -> Result<()>
    todo!()
}
fn step_in(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fn(err) -> ()) -> Result<()>
    todo!()
}
fn step_out(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fn(err) -> ()) -> Result<()>
    todo!()
}
fn step_over(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fn(err) -> ()) -> Result<()>
    todo!()
}
fn pause(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn() -> Result<()>
    todo!()
}
fn get_reg_value(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // fn(reg: String) -> Result<u16>
    // reg here can be R0-7, PC, PSR, MCR
    todo!()
}
fn set_reg_value(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // fn(reg: String, value: u16) -> Result<()>
    // reg here can be R0-7, PC, PSR, MCR
    todo!()
}
fn get_mem_value(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // fn (addr: u16) -> Result<u16>
    todo!()
}
fn set_mem_value(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // fn (addr: u16, value: u16) -> Result<()>
    todo!()
}
fn get_mem_line(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn(addr: u16) -> Result<String>
    todo!()
}
fn set_mem_line(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn(addr: u16, value: String) -> Result<()>
    todo!()
}
fn clear_input(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn() -> ()
    todo!()
}

fn add_input(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn(input: string) -> Result<()>
    // string is supposed to be char, though
    todo!()
}

fn set_breakpoint(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn(addr: u16) -> Result<()>
    todo!()
}

fn remove_breakpoint(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn(addr: u16) -> Result<()>
    todo!()
}

fn get_inst_exec_count(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // fn() -> Result<usize>
    // I have no idea what this does
    todo!()
}

fn did_hit_breakpoint(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    // fn() -> Result<bool>
    todo!()
}

fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(cx.string("hello node"))
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("hello", hello)?;
    cx.export_function("Init", init)?;
    cx.export_function("ConvertBin", convert_bin)?;
    cx.export_function("Assemble", assemble)?;
    cx.export_function("GetCurrSymTable", get_curr_sym_table)?;
    cx.export_function("SetEnableLiberalAsm", set_enable_liberal_asm)?;
    cx.export_function("LoadObjectFile", load_object_file)?;
    cx.export_function("RestartMachine", restart_machine)?;
    cx.export_function("ReinitializeMachine", reinitialize_machine)?;
    cx.export_function("RandomizeMachine", randomize_machine)?;
    cx.export_function("Run", run)?;
    cx.export_function("RunUntilHalt", run_until_halt)?;
    cx.export_function("StepIn", step_in)?;
    cx.export_function("StepOut", step_out)?;
    cx.export_function("StepOver", step_over)?;
    cx.export_function("Pause", pause)?;
    cx.export_function("GetRegValue", get_reg_value)?;
    cx.export_function("SetRegValue", set_reg_value)?;
    cx.export_function("GetMemValue", get_mem_value)?;
    cx.export_function("SetMemValue", set_mem_value)?;
    cx.export_function("GetMemLine", get_mem_line)?;
    cx.export_function("SetMemLine", set_mem_line)?;
    cx.export_function("SetIgnorePrivilege", set_ignore_privilege)?;
    cx.export_function("ClearInput", clear_input)?;
    cx.export_function("AddInput", add_input)?;
    cx.export_function("GetAndClearOutput", get_and_clear_output)?;
    cx.export_function("ClearOutput", clear_output)?;
    cx.export_function("SetBreakpoint", set_breakpoint)?;
    cx.export_function("RemoveBreakpoint", remove_breakpoint)?;
    cx.export_function("GetInstExecCount", get_inst_exec_count)?;
    cx.export_function("DidHitBreakpoint", did_hit_breakpoint)?;
    Ok(())
}
