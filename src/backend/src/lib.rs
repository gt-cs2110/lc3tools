mod err;
mod sim;
mod cast;

use std::collections::{HashMap, VecDeque};
use std::io::Write;
use std::ops::Range;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::{Arc, LazyLock, Mutex, MutexGuard, RwLock, RwLockWriteGuard};

use cast::{ResultExtJs, TryIntoJsValue};
use lc3_ensemble::asm::{assemble_debug, ObjectFile, SourceInfo, SymbolTable};
use lc3_ensemble::ast::asm::try_disassemble_line;
use lc3_ensemble::ast::Reg::{R0, R1, R2, R3, R4, R5, R6, R7};
use lc3_ensemble::parse::parse_ast;
use lc3_ensemble::sim::debug::Breakpoint;
use lc3_ensemble::sim::io::BufferedIO;
use lc3_ensemble::sim::mem::{MachineInitStrategy, Word};
use lc3_ensemble::sim::{MemAccessCtx, SimErr, SimFlags, Simulator};
use neon::prelude::*;
use err::{error_reporter, io_reporter, simple_reporter};
use sim::{SimAccessError, SimController};

static INPUT_BUFFER: LazyLock<Arc<RwLock<VecDeque<u8>>>> = LazyLock::new(Arc::default);
static PRINT_BUFFER: LazyLock<Arc<RwLock<Vec<u8>>>> = LazyLock::new(Arc::default);

/// Creates a write guard to [`INPUT_BUFFER`].
fn input_writer() -> RwLockWriteGuard<'static, VecDeque<u8>> {
    INPUT_BUFFER.write()
        .unwrap_or_else(|e| e.into_inner())
}
/// Creates a write guard to [`PRINT_BUFFER`].
fn print_writer() -> RwLockWriteGuard<'static, Vec<u8>> {
    PRINT_BUFFER.write()
        .unwrap_or_else(|e| e.into_inner())
}
/// Creates a BufferedIO value that can be set as the Simulator's IO.
fn get_buffered_io() -> BufferedIO {
    BufferedIO::with_bufs(Arc::clone(&INPUT_BUFFER), Arc::clone(&PRINT_BUFFER))
}

fn sim_contents() -> MutexGuard<'static, SimPageContents> {
    static SIM_CONTENTS: LazyLock<Mutex<SimPageContents>> = LazyLock::new(|| {
        Mutex::new(SimPageContents {
            controller: SimController::new(),
            obj_file: None,
            sim_flags: Default::default(),
            mem_lines: Default::default()
        })
    });
    
    match SIM_CONTENTS.lock() {
        Ok(g) => g,
        Err(e) => {
            // Errors don't put the page contents into an invalid state,
            // so it should be okay to just do this
            SIM_CONTENTS.clear_poison();
            e.into_inner()
        }
    }
}

fn get_create_strategy(zeroed: bool) -> MachineInitStrategy {
    match zeroed {
        true => MachineInitStrategy::Known { value: 0 },
        false => MachineInitStrategy::Unseeded
    }
}

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
                    Some((addr, src_info.source()[span].to_string()))
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

struct SimPageContents {
    controller: SimController,
    obj_file: Option<ObjectFile>,
    mem_lines: HashMap<u16, String>,
    sim_flags: SimFlags
}
impl SimPageContents {
    /// Updates the simulator flags for the controller.
    fn update_sim_flags(&mut self, f: impl FnOnce(&mut SimFlags)) {
        f(&mut self.sim_flags);
        // Apply immediately to simulator if possible.
        // If not, this will be applied on execution or reset.
        let Ok(sim) = self.controller.simulator() else { return };
        sim.flags = self.sim_flags;
    }
    /// Executes and applies flags.
    fn execute<T>(&mut self, 
        exec: impl FnOnce(&mut Simulator) -> T + Send + 'static,
        close: impl FnOnce(T) + Send + 'static
    ) -> Result<(), SimAccessError> {
        self.controller.execute(exec, close, self.sim_flags)
    }

    fn read_mem(&mut self, cx: &mut FunctionContext, addr: u16) -> NeonResult<Word> {
        let simulator = self.controller.simulator().or_throw(cx)?;

        simulator.read_mem(addr, MemAccessCtx::omnipotent()).or_throw(cx)
    }

    fn write_mem(&mut self, cx: &mut FunctionContext, addr: u16, word: u16) -> NeonResult<()> {
        let simulator = self.controller.simulator().or_throw(cx)?;

        simulator.write_mem(addr, Word::new_init(word), MemAccessCtx::omnipotent()).or_throw(cx)?;

        Ok(())
    }
    fn get_mem_line(&self, addr: u16) -> &str {
        self.mem_lines.get(&addr).map_or("", |s| s)
    }
    fn set_mem_line(&mut self, addr: u16, value: u16) {
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
    fn load_obj_file(&mut self, obj: ObjectFile) {
        let flags = SimFlags {
            machine_init: get_create_strategy(false),
            ..self.sim_flags
        };
        let sim = self.controller.reset(flags);
        sim.open_io(get_buffered_io());
        sim.load_obj_file(&obj);

        // Set mem lines:
        self.mem_lines.clear();
        add_mem_lines_from_obj(&mut self.mem_lines, lc3_ensemble::sim::_os_obj_file());
        add_mem_lines_from_obj(&mut self.mem_lines, &obj);
        //
        
        self.obj_file.replace(obj);
    }
    fn reset_machine(&mut self, init: MachineInitStrategy) {
        self.sim_flags.machine_init = init;
        let sim = self.controller.reset(self.sim_flags);
        sim.open_io(get_buffered_io());
        self.obj_file.take();
        self.mem_lines.clear();
    }
    fn get_sym_source(&self) -> Option<(&SymbolTable, &SourceInfo)> {
        get_sym_source_from_obj(self.obj_file.as_ref()?)
    }
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
    let ignore_privilege = cx.argument::<JsBoolean>(0)?.value(&mut cx);
    sim_contents().update_sim_flags(|f| f.ignore_privilege = ignore_privilege);

    Ok(cx.undefined())
}
fn set_pause_on_fatal_trap(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn(enable: bool) -> Result<()>
    // the boolean flag is the pause_on_fatal_trap flag.
    // if pause_on_fatal_trap is true, we're applying "virtual" mode
    // i.e., these are inverses of each other
    let use_real_traps = !cx.argument::<JsBoolean>(0)?.value(&mut cx);
    sim_contents().update_sim_flags(|f| f.use_real_traps = use_real_traps);
    
    Ok(cx.undefined())
}

//--------- CONSOLE FUNCTIONS ---------//

fn get_and_clear_output(mut cx: FunctionContext) -> JsResult<JsString> {
    // fn() -> Result<String>
    let bytes = std::mem::take(&mut *print_writer());
    let string = String::from_utf8_lossy(&bytes);
    Ok(cx.string(string))
}

fn clear_output(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn() -> Result<()>
    print_writer().clear();
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
        .map_err(|e| error_reporter(&e, in_path, &src).report_and_throw(&mut *print_writer(), &mut cx))?;
    let asm = assemble_debug(ast, &src)
        .map_err(|e| error_reporter(&e, in_path, &src).report_and_throw(&mut *print_writer(), &mut cx))?;
    
    std::fs::write(&out_path, asm.write_bytes())
        .map_err(|e| io_reporter(&e, in_path).report_and_throw(&mut *print_writer(), &mut cx))?;

    writeln!(print_writer(), "successfully assembled {} into {}", in_path.display(), out_path.display()).unwrap();
    Ok(cx.undefined())
}

//--------- SIMULATOR FUNCTIONS ---------//

fn get_curr_sym_table(mut cx: FunctionContext) -> JsResult<JsObject> {
    // fn () -> Result<Object>
    let obj = cx.empty_object();

    let contents = sim_contents();
    let Some(obj_file) = contents.obj_file.as_ref() else { return Ok(obj) };
    let Some(sym) = obj_file.symbol_table() else { return Ok(obj) };
    for (label, addr) in sym.label_iter() {
        let key = cx.number(addr);
        let val = cx.string(label);
        obj.set(&mut cx, key, val)?;
    }
    Ok(obj)
}
fn load_object_file(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fp: string) -> Result<()>
    let fp = cx.argument::<JsString>(0)?.value(&mut cx);
    let in_path = AsRef::<Path>::as_ref(&fp);
    
    // should be unreachable cause frontend validates IO
    let bytes = std::fs::read(in_path).unwrap();
    
    let Some(obj) = ObjectFile::read_bytes(&bytes) else {
        return Err(io_reporter("malformed object file", in_path).report_and_throw(&mut *print_writer(), &mut cx));
    };

    sim_contents().load_obj_file(obj);
    Ok(cx.undefined())
}
fn restart_machine(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn () -> Result<()>
    // i'm not sure what the purpose of this function is

    Ok(cx.undefined())
}
fn reinitialize_machine(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn () -> Result<()>

    sim_contents().reset_machine(get_create_strategy(true));
    Ok(cx.undefined())
}
fn randomize_machine(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fn(err) -> ()) -> Result<()>

    sim_contents().reset_machine(get_create_strategy(false));
    Ok(cx.undefined())
}

/// Helper that handles the result of the simulation and sends the error (if it exists)  back to the JS thread.
fn finish_execution(channel: Channel, cb: Root<JsFunction>, result: Result<(), SimErr>) {
    channel.send(move |mut cx| {
        let this = cx.undefined();
        let arg = cx.undefined().as_value(&mut cx);

        if let Err(e) = result {
            let pc = sim_contents()
                .controller
                .simulator()
                .or_throw(&mut cx)?
                .prefetch_pc();
            
            simple_reporter(&format!("{e} (instruction x{pc:04X})"))
                .report(&mut *print_writer());
        }

        cb.into_inner(&mut cx)
            .call(&mut cx, this, vec![arg])?;

        Ok(())
    });
}

fn run(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fn(err) -> ()) -> Result<()>
    let channel = cx.channel();
    let done_cb = cx.argument::<JsFunction>(0)?.root(&mut cx);

    sim_contents().execute(
        Simulator::run,
        |result| finish_execution(channel, done_cb, result)
    ).or_throw(&mut cx)?;

    Ok(cx.undefined())
}
fn step_in(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fn(err) -> ()) -> Result<()>
    let channel = cx.channel();
    let done_cb = cx.argument::<JsFunction>(0)?.root(&mut cx);
    
    sim_contents()
        .execute(
            Simulator::step_in,
            |result| finish_execution(channel, done_cb, result)
        )
        .or_throw(&mut cx)
        .try_into_js(&mut cx)
}
fn step_out(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fn(err) -> ()) -> Result<()>
    let channel = cx.channel();
    let done_cb = cx.argument::<JsFunction>(0)?.root(&mut cx);
    
    sim_contents()
        .execute(
            Simulator::step_out,
            |result| finish_execution(channel, done_cb, result)
        )
        .or_throw(&mut cx)
        .try_into_js(&mut cx)
}
fn step_over(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fn(err) -> ()) -> Result<()>
    let channel = cx.channel();
    let done_cb = cx.argument::<JsFunction>(0)?.root(&mut cx);
    
    sim_contents()
        .execute(
            Simulator::step_over,
            |result| finish_execution(channel, done_cb, result)
        )
        .or_throw(&mut cx)
        .try_into_js(&mut cx)
}
fn pause(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    sim_contents().controller.pause()
        .or_throw(&mut cx)
        .try_into_js(&mut cx)
}

fn get_reg_value(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // fn(reg: String) -> Result<u16>
    // reg here can be R0-7, PC, PSR, MCR
    let reg = cx.argument::<JsString>(0)?.value(&mut cx);

    let mut sim_contents = sim_contents();
    let simulator = sim_contents.controller.simulator().or_throw(&mut cx)?;

    let value = match &*reg {
        "r0"  => simulator.reg_file[R0].get(),
        "r1"  => simulator.reg_file[R1].get(),
        "r2"  => simulator.reg_file[R2].get(),
        "r3"  => simulator.reg_file[R3].get(),
        "r4"  => simulator.reg_file[R4].get(),
        "r5"  => simulator.reg_file[R5].get(),
        "r6"  => simulator.reg_file[R6].get(),
        "r7"  => simulator.reg_file[R7].get(),
        "pc"  => simulator.pc,
        "psr" => simulator.psr().get(),
        "mcr" => {
            let mcr = simulator.mcr();
            if mcr.load(Ordering::Relaxed) { 0x8000 } else { 0x0000 }
        }
        reg => cx.throw_error(format!("undefined register {reg:?}"))?
    };
    std::mem::drop(sim_contents);
    
    Ok(cx.number(value))
}
fn set_reg_value(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn(reg: String, value: u16) -> Result<()>
    // reg here can be R0-7, PC, PSR, MCR
    let reg = cx.argument::<JsString>(0)?.value(&mut cx);
    let value = cx.argument::<JsNumber>(1)?.value(&mut cx) as u16;

    let mut sim_contents = sim_contents();
    let simulator = sim_contents.controller.simulator().or_throw(&mut cx)?;

    match &*reg {
        "r0"  => simulator.reg_file[R0].set(value),
        "r1"  => simulator.reg_file[R1].set(value),
        "r2"  => simulator.reg_file[R2].set(value),
        "r3"  => simulator.reg_file[R3].set(value),
        "r4"  => simulator.reg_file[R4].set(value),
        "r5"  => simulator.reg_file[R5].set(value),
        "r6"  => simulator.reg_file[R6].set(value),
        "r7"  => simulator.reg_file[R7].set(value),
        "pc"  => simulator.pc = value,
        "psr" => sim_contents.write_mem(&mut cx, 0xFFFC, value)?,
        "mcr" => sim_contents.write_mem(&mut cx, 0xFFFE, value)?,
        reg => cx.throw_error(format!("undefined register {reg:?}"))?
    };
    std::mem::drop(sim_contents);
    
    Ok(cx.undefined())
}
fn get_mem_value(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // fn (addr: u16) -> Result<u16>
    let addr = cx.argument::<JsNumber>(0)?.value(&mut cx) as u16;

    let mut sim_contents = sim_contents();
    let value = sim_contents.read_mem(&mut cx, addr)?.get();

    Ok(cx.number(value))
}
fn set_mem_value(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (addr: u16, value: u16) -> Result<()>
    let addr  = cx.argument::<JsNumber>(0)?.value(&mut cx) as u16;
    let value = cx.argument::<JsNumber>(1)?.value(&mut cx) as u16;
    
    let mut sim_contents = sim_contents();
    sim_contents.write_mem(&mut cx, addr, value).try_into_js(&mut cx)
}
fn take_mem_changes(mut cx: FunctionContext) -> JsResult<JsArray> {
    let mut sim_contents = sim_contents();
    let simulator = sim_contents.controller.simulator().or_throw(&mut cx)?;

    let changes: Vec<_> = simulator.observer.take_mem_changes().collect();
    // Update mem lines:
    for &addr in &changes {
        let value = sim_contents.read_mem(&mut cx, addr)?;
        sim_contents.set_mem_line(addr, value.get());
    }
    // Return mem lines:
    changes.try_into_js(&mut cx)
}
fn get_mem_line(mut cx: FunctionContext) -> JsResult<JsString> {
    // fn(addr: u16, force_recompute: bool) -> Result<String>
    let addr = cx.argument::<JsNumber>(0)?.value(&mut cx) as u16;

    let sim_contents = sim_contents();
    let string = sim_contents.get_mem_line(addr);
    
    Ok(cx.string(string))
}
fn set_mem_line(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn(addr: u16, value: String) -> Result<()>
    // TODO: implement
    Ok(cx.undefined())
}
fn clear_input(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn() -> ()
    input_writer().clear();
    Ok(cx.undefined())
}

fn add_input(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn(input: string) -> Result<()>
    // string is supposed to be char, though
    let input = cx.argument::<JsString>(0)?.value(&mut cx);
    
    // ignore input requests unless they're happening while the sim is running
    if let Err(sim::SimAccessError::NotAvailable) = sim_contents().controller.simulator() {
        let &[ch] = input.as_bytes() else {
            return cx.throw_error("more than one byte was sent at once");
        };
        input_writer().push_back(ch);
    }

    Ok(cx.undefined())
}

fn set_breakpoint(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    // fn(addr: u16) -> Result<bool>
    let addr = cx.argument::<JsNumber>(0)?.value(&mut cx) as u16;
    let value = sim_contents()
        .controller
        .simulator()
        .or_throw(&mut cx)?
        .breakpoints
        .insert(Breakpoint::PC(addr));
    Ok(cx.boolean(value))
}

fn remove_breakpoint(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    // fn(addr: u16) -> Result<bool>
    let addr = cx.argument::<JsNumber>(0)?.value(&mut cx) as u16;
    let result = sim_contents()
        .controller
        .simulator()
        .or_throw(&mut cx)?
        .breakpoints
        .remove(&Breakpoint::PC(addr));

    Ok(cx.boolean(result))
}

fn get_inst_exec_count(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // fn() -> Result<usize>
    // I have no idea what this does
    Ok(cx.number(0))
}

fn did_hit_breakpoint(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    // fn() -> Result<bool>
    let hit = sim_contents()
        .controller
        .simulator()
        .or_else(|e| cx.throw_error(e.to_string()))?
        .hit_breakpoint();
    
    Ok(cx.boolean(hit))
}
fn is_sim_running(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let hit = sim_contents()
        .controller
        .simulator()
        .is_err();

    Ok(cx.boolean(hit))
}
fn get_label_source_range(mut cx: FunctionContext) -> JsResult<JsValue> {
    let label = cx.argument::<JsString>(0)?.value(&mut cx);
    
    let sim_contents = sim_contents();
    'get_line: {
        let Some((sym, src_info)) = sim_contents.get_sym_source() else { break 'get_line };
        let Some(Range { start, end }) = sym.get_label_source(&label) else { break 'get_line };
        let (slno, scno) = src_info.get_pos_pair(start);
        let (elno, ecno) = src_info.get_pos_pair(end);

        return [slno, scno, elno, ecno]
            .try_into_js(&mut cx)
            .map(|e| e.upcast());
    }
    Ok(cx.undefined().upcast())

}
fn get_addr_source_range(mut cx: FunctionContext) -> JsResult<JsValue> {
    let addr = cx.argument::<JsNumber>(0)?.value(&mut cx) as u16;

    let sim_contents = sim_contents();
    'get_line: {
        let Some((sym, src_info)) = sim_contents.get_sym_source() else { break 'get_line };
        let Some(lno) = sym.rev_lookup_line(addr) else { break 'get_line };
        let Some(Range { start, end }) = src_info.line_span(lno) else { break 'get_line };
        let (slno, scno) = src_info.get_pos_pair(start);
        let (elno, ecno) = src_info.get_pos_pair(end);

        return [slno, scno, elno, ecno]
            .try_into_js(&mut cx)
            .map(|e| e.upcast());
    }
    Ok(cx.undefined().upcast())
}
#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("init", init)?;
    cx.export_function("convertBin", convert_bin)?;
    cx.export_function("assemble", assemble)?;
    cx.export_function("getCurrSymTable", get_curr_sym_table)?;
    cx.export_function("setEnableLiberalAsm", set_enable_liberal_asm)?;
    cx.export_function("loadObjectFile", load_object_file)?;
    cx.export_function("restartMachine", restart_machine)?;
    cx.export_function("reinitializeMachine", reinitialize_machine)?;
    cx.export_function("randomizeMachine", randomize_machine)?;
    cx.export_function("run", run)?;
    cx.export_function("stepIn", step_in)?;
    cx.export_function("stepOut", step_out)?;
    cx.export_function("stepOver", step_over)?;
    cx.export_function("pause", pause)?;
    cx.export_function("getRegValue", get_reg_value)?;
    cx.export_function("setRegValue", set_reg_value)?;
    cx.export_function("getMemValue", get_mem_value)?;
    cx.export_function("setMemValue", set_mem_value)?;
    cx.export_function("getMemLine", get_mem_line)?;
    cx.export_function("setMemLine", set_mem_line)?;
    cx.export_function("takeMemChanges", take_mem_changes)?;
    cx.export_function("setIgnorePrivilege", set_ignore_privilege)?;
    cx.export_function("setPauseOnFatalTrap", set_pause_on_fatal_trap)?;
    cx.export_function("clearInput", clear_input)?;
    cx.export_function("addInput", add_input)?;
    cx.export_function("getAndClearOutput", get_and_clear_output)?;
    cx.export_function("clearOutput", clear_output)?;
    cx.export_function("setBreakpoint", set_breakpoint)?;
    cx.export_function("removeBreakpoint", remove_breakpoint)?;
    cx.export_function("getInstExecCount", get_inst_exec_count)?;
    cx.export_function("didHitBreakpoint", did_hit_breakpoint)?;
    cx.export_function("isSimRunning", is_sim_running)?;
    cx.export_function("getLabelSourceRange", get_label_source_range)?;
    cx.export_function("getAddrSourceRange", get_addr_source_range)?;
    Ok(())
}
