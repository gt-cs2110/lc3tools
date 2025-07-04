mod err;
mod sim;
mod cast;
mod obj;

use std::collections::HashMap;
use std::io::Write;
use std::ops::{Range, RangeBounds};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::{LazyLock, Mutex, MutexGuard};

use cast::{IntoJsValue, ResultExtJs, TryIntoJsValue};
use lc3_ensemble::asm::encoding::{BinaryFormat, ObjFileFormat, TextFormat};
use lc3_ensemble::asm::{assemble_debug, ObjectFile};
use lc3_ensemble::ast::Reg::{R0, R1, R2, R3, R4, R5, R6, R7};
use lc3_ensemble::parse::parse_ast;
use lc3_ensemble::sim::debug::Breakpoint;
use lc3_ensemble::sim::device::ExternalDevice;
use lc3_ensemble::sim::mem::MachineInitStrategy;
use lc3_ensemble::sim::{SimErr, Simulator};
use neon::prelude::*;
use err::Reporter;
use obj::ObjContents;
use owo_colors::OwoColorize;
use sim::SimController;

static CONTROLLER: LazyLock<Mutex<SimController>> = LazyLock::new(Mutex::default);
static SIM_CONTENTS: LazyLock<Mutex<ObjContents>> = LazyLock::new(Mutex::default);

fn obj_contents() -> MutexGuard<'static, ObjContents> {
    SIM_CONTENTS.lock().unwrap_or_else(|e| e.into_inner())
}
fn controller() -> MutexGuard<'static, SimController> {
    CONTROLLER.lock().unwrap_or_else(|e| e.into_inner())
}
pub fn deserialize_obj_file(bytes: Vec<u8>) -> Option<ObjectFile> {
    match String::from_utf8(bytes) {
        Ok(s) => TextFormat::deserialize(&s),
        Err(e) => BinaryFormat::deserialize(e.as_bytes()),
    }
}

/// Get the common ancestor of all listed paths.
fn common_ancestor<'p>(p: impl IntoIterator<Item=&'p Path>) -> &'p Path {
    fn common_ancestor2<'p>(p1: &'p Path, p2: &'p Path) -> &'p Path {
        p1.ancestors()
            .find(|ancestor| p2.starts_with(ancestor))
            .unwrap_or_else(|| Path::new(""))
    }

    p.into_iter()
        .reduce(common_ancestor2)
        .unwrap_or_else(|| Path::new(""))
}
/// Get a path display, assuming `p` is relative to `ancestor`.
fn display_path<'p>(p: &'p Path, ancestor: &'p Path) -> impl std::fmt::Display + 'p {
    p.strip_prefix(ancestor).unwrap_or(p).display()
}

fn reset_machine(zeroed: bool) {
    let init = match zeroed {
        true => MachineInitStrategy::Known { value: 0 },
        false => MachineInitStrategy::Unseeded
    };

    let mut controller = controller();
    controller.update_flags(|f| f.machine_init = init);
    controller.reset();

    obj_contents().clear();
}
fn load_obj_file(obj: ObjectFile) -> Result<(), SimErr> {
    reset_machine(false);
    let mut controller = controller();
    
    controller.simulator()
        .unwrap_or_else(|_| panic!("simulator should've been idle after reset"))
        .load_obj_file(&obj)?;

    obj_contents().load_contents(obj);
    Ok(())
}
//--------- CONFIG FUNCTIONS ---------//
fn set_ignore_privilege(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn(enable: bool) -> Result<()>
    let ignore_privilege = cx.argument::<JsBoolean>(0)?.value(&mut cx);
    controller().update_flags(|f| f.ignore_privilege = ignore_privilege);

    Ok(cx.undefined())
}
fn set_pause_on_fatal_trap(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn(enable: bool) -> Result<()>
    // the boolean flag is the pause_on_fatal_trap flag.
    // if pause_on_fatal_trap is true, we're applying "virtual" mode
    // i.e., these are inverses of each other
    let use_real_traps = !cx.argument::<JsBoolean>(0)?.value(&mut cx);
    controller().update_flags(|f| f.use_real_traps = use_real_traps);
    
    Ok(cx.undefined())
}

//--------- CONSOLE FUNCTIONS ---------//

fn get_and_clear_output(mut cx: FunctionContext) -> JsResult<JsString> {
    // fn() -> Result<String>
    let bytes = std::mem::take(&mut *controller().output_buf());
    let string = String::from_utf8_lossy(&bytes);
    Ok(cx.string(string))
}

fn clear_output(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn() -> Result<()>
    controller().output_buf().clear();
    Ok(cx.undefined())
}

//--------- EDITOR/ASSEMBLER FUNCTIONS ---------//
fn assemble(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fp: String) -> Result<()>
    let in_path: PathBuf = cx.argument::<JsString>(0)?.value(&mut cx).into();
    let out_path = in_path.with_extension("obj");
    
    // should be unreachable cause frontend validates IO
    let src = std::fs::read_to_string(&in_path).or_throw(&mut cx)?;

    let ast = parse_ast(&src)
        .map_err(|e| Reporter::ensemble(&e, &in_path, &src).report_and_throw(&mut controller().output_buf(), &mut cx))?;
    let obj = assemble_debug(ast, &src)
        .map_err(|e| Reporter::ensemble(&e, &in_path, &src).report_and_throw(&mut controller().output_buf(), &mut cx))?;
    
    std::fs::write(&out_path, TextFormat::serialize(&obj))
        .map_err(|e| Reporter::io(&e, &out_path).report_and_throw(&mut controller().output_buf(), &mut cx))?;

    let ancestor = common_ancestor([&*in_path, &*out_path]);
    let rel_in = display_path(&in_path, ancestor);
    let rel_out = display_path(&out_path, ancestor);
    writeln!(controller().output_buf(), "successfully assembled {} into {}", rel_in.underline(), rel_out.underline())
        .unwrap();
    Ok(cx.undefined())
}

fn link(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fp: String[], out: String) -> Result<()>
    let out: PathBuf = cx.argument::<JsString>(1)?.value(&mut cx).into();

    let file_paths: Vec<PathBuf> = cx.argument::<JsArray>(0)?
        .to_vec(&mut cx)?
        .into_iter()
        .map(|e| e.downcast_or_throw::<JsString, _>(&mut cx).map(|s| s.value(&mut cx).into()))
        .collect::<Result<_, _>>()?;

    let mut result_obj = ObjectFile::empty();
    for fp in &file_paths {
        // Parse object file:
        let src = std::fs::read_to_string(fp).or_throw(&mut cx)?;
        let obj = deserialize_obj_file(src.into_bytes())
            .ok_or_else(|| {
                Reporter::io("cannot deserialize object file", fp)
                    .report_and_throw(&mut controller().output_buf(), &mut cx)
            })?;

        // Link to current result obj:
        result_obj = ObjectFile::link(result_obj, obj)
            .map_err(|e| Reporter::simple(&e).report_and_throw(&mut controller().output_buf(), &mut cx))?;
    }
    std::fs::write(&out, TextFormat::serialize(&result_obj)).or_throw(&mut cx)?;

    let ancestor = common_ancestor(file_paths.iter().chain([&out]).map(|p| &**p));
    let in_fs = file_paths.iter()
        .map(|p| display_path(p, ancestor).underline().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let out_f = display_path(&out, ancestor);

    writeln!(controller().output_buf(), "successfully linked object files [{}] to {}", in_fs.underline(), out_f.underline()).unwrap();

    Ok(cx.undefined())
}
//--------- SIMULATOR FUNCTIONS ---------//

fn get_curr_sym_table(mut cx: FunctionContext) -> JsResult<JsObject> {
    // fn () -> Result<Object>
    
    let contents = obj_contents();
    let mut map = HashMap::new();

    if let Some((sym, _)) = contents.get_sym_source() {
        map.extend(sym.label_iter().map(|(label, addr, _)| (addr, label)));
    }

    map.try_into_js(&mut cx)
}
fn load_object_file(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fp: string) -> Result<()>
    let in_path: PathBuf = cx.argument::<JsString>(0)?.value(&mut cx).into();
    
    // should be unreachable cause frontend validates IO
    let bytes = std::fs::read(&in_path).or_throw(&mut cx)?;
    
    let Some(obj) = deserialize_obj_file(bytes) else {
        return Err(
            Reporter::io("malformed object file", &in_path)
                .report_and_throw(&mut controller().output_buf(), &mut cx)
        );
    };
    
    match load_obj_file(obj) {
        Ok(_) => Ok(cx.undefined()),
        Err(e) => Err(Reporter::simple(&e).report_and_throw(&mut controller().output_buf(), &mut cx)),
    }
}
fn reinitialize_machine(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn () -> Result<()>

    reset_machine(true);
    Ok(cx.undefined())
}
fn randomize_machine(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fn(err) -> ()) -> Result<()>

    reset_machine(false);
    Ok(cx.undefined())
}

/// Helper that handles the result of the simulation and sends the error (if it exists)  back to the JS thread.
fn finish_execution(channel: Channel, cb: Root<JsFunction>, result: Result<(), SimErr>) {
    channel.send(move |mut cx| {
        let this = cx.undefined();
        let arg = cx.undefined().upcast();

        if let Err(e) = result {
            let pc = controller().simulator()
                .or_throw(&mut cx)?
                .prefetch_pc();
            
            Reporter::simple(&format!("{e} (PC: x{pc:04X})"))
                .report(&mut controller().output_buf());
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

    controller().execute(
        Simulator::run,
        |result| finish_execution(channel, done_cb, result)
    ).or_throw(&mut cx)?;

    Ok(cx.undefined())
}
fn step_in(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fn(err) -> ()) -> Result<()>
    let channel = cx.channel();
    let done_cb = cx.argument::<JsFunction>(0)?.root(&mut cx);
    
    controller().execute(
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
    
    controller().execute(
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
    
    controller().execute(
            Simulator::step_over,
            |result| finish_execution(channel, done_cb, result)
        )
        .or_throw(&mut cx)
        .try_into_js(&mut cx)
}
fn pause(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    controller().pause();
    Ok(cx.undefined())
}

fn get_reg_value(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // fn(reg: String) -> Result<u16>
    // reg here can be R0-7, PC, PSR, MCR
    let reg = cx.argument::<JsString>(0)?.value(&mut cx);

    let mut controller = controller();
    let simulator = controller.simulator().or_throw(&mut cx)?;

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
    
    Ok(cx.number(value))
}
fn set_reg_value(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn(reg: String, value: u16) -> Result<()>
    // reg here can be R0-7, PC, PSR, MCR
    let reg = cx.argument::<JsString>(0)?.value(&mut cx);
    let value = cx.argument::<JsNumber>(1)?.value(&mut cx) as u16;

    let mut controller = controller();
    let simulator = controller.simulator().or_throw(&mut cx)?;

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
        "psr" => controller.write_mem(0xFFFC, value).or_throw(&mut cx)?,
        "mcr" => controller.write_mem(0xFFFE, value).or_throw(&mut cx)?,
        reg => cx.throw_error(format!("undefined register {reg:?}"))?
    };
    
    Ok(cx.undefined())
}
fn get_mem_value(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // fn (addr: u16) -> Result<u16>
    let addr = cx.argument::<JsNumber>(0)?.value(&mut cx) as u16;

    let value = controller().read_mem(addr)
        .or_throw(&mut cx)?
        .get();

    Ok(cx.number(value))
}
fn set_mem_value(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (addr: u16, value: u16) -> Result<()>
    let addr  = cx.argument::<JsNumber>(0)?.value(&mut cx) as u16;
    let value = cx.argument::<JsNumber>(1)?.value(&mut cx) as u16;
    
    controller().write_mem(addr, value)
        .or_throw(&mut cx)
        .try_into_js(&mut cx)
}
fn take_mem_changes(mut cx: FunctionContext) -> JsResult<JsArray> {
    let mut controller = controller();
    let mut contents = obj_contents();
    
    let simulator = controller.simulator().or_throw(&mut cx)?;
    let changes: Vec<_> = simulator.observer
        .take_mem_accesses()
        .filter_map(|(addr, access)| access.modified().then_some(addr))
        .collect();
    // Update mem lines:
    for &addr in &changes {
        let value = controller.read_mem(addr).or_throw(&mut cx)?;
        contents.set_mem_line(addr, value.get());
    }
    // Return mem lines:
    changes.try_into_js(&mut cx)
}
fn get_mem_line(mut cx: FunctionContext) -> JsResult<JsString> {
    // fn(addr: u16, force_recompute: bool) -> Result<String>
    let addr = cx.argument::<JsNumber>(0)?.value(&mut cx) as u16;

    let contents = obj_contents();
    let string = contents.get_mem_line(addr);
    
    Ok(cx.string(string))
}
fn clear_input(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn() -> ()
    controller().input_buf().clear();
    Ok(cx.undefined())
}

fn add_input(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn(input: string) -> Result<()>
    // string is supposed to be char, though
    let input = cx.argument::<JsString>(0)?.value(&mut cx);
    let mut controller = controller();

    // ignore input requests unless they're happening while the sim is running
    if controller.is_running() {
        let &[ch] = input.as_bytes() else {
            return cx.throw_error("more than one byte was sent at once");
        };
        controller.input_buf().push_back(ch);
    }

    Ok(cx.undefined())
}

fn get_breakpoints(mut cx: FunctionContext) -> JsResult<JsArray> {
    let mut controller = controller();
    let sim = controller.simulator().or_throw(&mut cx)?;
    
    let mut breakpoints: Vec<_> = sim.breakpoints.iter()
        .filter_map(|bp| match *bp {
            Breakpoint::PC(pc) => Some(pc),
            _ => None
        })
        .collect();
    breakpoints.sort();

    breakpoints.try_into_js(&mut cx)
}
fn set_breakpoint(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    // fn(addr: u16) -> Result<bool>
    let addr = cx.argument::<JsNumber>(0)?.value(&mut cx) as u16;
    
    let mut controller = controller();
    let sim = controller.simulator().or_throw(&mut cx)?;
    let value = sim.breakpoints.insert(Breakpoint::PC(addr));
    
    Ok(cx.boolean(value))
}

fn remove_breakpoint(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    // fn(addr: u16) -> Result<bool>
    let addr = cx.argument::<JsNumber>(0)?.value(&mut cx) as u16;

    let mut controller = controller();
    let sim = controller.simulator().or_throw(&mut cx)?;
    let result = sim.breakpoints.remove(&Breakpoint::PC(addr));

    Ok(cx.boolean(result))
}

fn did_hit_breakpoint(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    // fn() -> Result<bool>
    let mut controller = controller();
    let hit = controller.simulator()
        .or_throw(&mut cx)?
        .hit_breakpoint();
    
    Ok(cx.boolean(hit))
}

fn get_frame_number(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let mut controller = controller();
    let fno = controller.simulator()
        .or_throw(&mut cx)?
        .frame_stack
        .len();

    fno.try_into_js(&mut cx)
}

fn is_sim_running(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    // fn() -> bool
    Ok(cx.boolean(controller().is_running()))
}
fn get_label_source_range(mut cx: FunctionContext) -> JsResult<JsValue> {
    // fn(label: String) -> Result<Option<usize>>
    let label = cx.argument::<JsString>(0)?.value(&mut cx);
    
    let contents = obj_contents();
    'get_line: {
        let Some((sym, src_info)) = contents.get_sym_source() else { break 'get_line };
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
    // fn(addr: u16) -> Result<Option<usize>>
    let addr = cx.argument::<JsNumber>(0)?.value(&mut cx) as u16;

    let contents = obj_contents();
    'get_line: {
        let Some((sym, src_info)) = contents.get_sym_source() else { break 'get_line };
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

fn get_timer_remaining(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // fn() -> u32
    let controller = controller();
    let timer = controller.timer();

    Ok(timer.get_remaining().into_js(&mut cx))
}
fn set_timer_status(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn(enabled: bool)
    let enabled = cx.argument::<JsBoolean>(0)?.value(&mut cx);

    let controller = controller();
    controller.timer().enabled = enabled;
    Ok(cx.undefined())
}
fn reset_timer(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn()
    let controller = controller();
    controller.timer().io_reset();

    Ok(cx.undefined())
}

fn get_timer_vect(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // fn() -> u8
    let controller = controller();
    let timer = controller.timer();
    Ok(cx.number(timer.vect))
}
fn get_timer_priority(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // fn() -> u8
    let controller = controller();
    let timer = controller.timer();
    Ok(cx.number(timer.priority.clamp(0, 7)))
}
fn get_timer_max(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // fn() -> u32
    let controller = controller();
    let timer = controller.timer();
    let std::ops::Bound::Included(&max) = timer.get_range().start_bound() else { unreachable!("definition for timer") };
    Ok(cx.number(max))
}
fn set_timer_vect(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn(vect: u8)
    let vect = cx.argument::<JsNumber>(0)?.value(&mut cx) as u8;

    let controller = controller();
    controller.timer().vect = vect;
    Ok(cx.undefined())
}
fn set_timer_priority(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn(priority: u8)
    let priority = cx.argument::<JsNumber>(0)?.value(&mut cx) as u8;
    if !(0..8).contains(&priority) { 
        cx.throw_error("lc3.setTimerPriority: priority was not within the range 0-7")?;
    }

    let controller = controller();
    controller.timer().priority = priority;
    Ok(cx.undefined())
}
fn set_timer_max(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn(min: u32, max?: u32)
    let min = cx.argument::<JsNumber>(0)?.value(&mut cx) as u32;
    let max = match cx.argument_opt(1) {
        Some(n) => n.downcast_or_throw::<JsNumber, _>(&mut cx)?.value(&mut cx) as u32,
        None => min,
    };

    let controller = controller();
    let mut timer = controller.timer();
    timer.set_range(min..=max);

    Ok(cx.undefined())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("assemble", assemble)?;
    cx.export_function("link", link)?;
    cx.export_function("getCurrSymTable", get_curr_sym_table)?;
    cx.export_function("setIgnorePrivilege", set_ignore_privilege)?;
    cx.export_function("setPauseOnFatalTrap", set_pause_on_fatal_trap)?;
    cx.export_function("loadObjectFile", load_object_file)?;
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
    cx.export_function("takeMemChanges", take_mem_changes)?;
    cx.export_function("clearInput", clear_input)?;
    cx.export_function("addInput", add_input)?;
    cx.export_function("getAndClearOutput", get_and_clear_output)?;
    cx.export_function("clearOutput", clear_output)?;
    cx.export_function("getBreakpoints", get_breakpoints)?;
    cx.export_function("setBreakpoint", set_breakpoint)?;
    cx.export_function("removeBreakpoint", remove_breakpoint)?;
    cx.export_function("didHitBreakpoint", did_hit_breakpoint)?;
    cx.export_function("getFrameNumber", get_frame_number)?;
    cx.export_function("isSimRunning", is_sim_running)?;
    cx.export_function("getLabelSourceRange", get_label_source_range)?;
    cx.export_function("getAddrSourceRange", get_addr_source_range)?;
    cx.export_function("getTimerRemaining", get_timer_remaining)?;
    cx.export_function("setTimerStatus", set_timer_status)?;
    cx.export_function("resetTimer", reset_timer)?;
    cx.export_function("getTimerVect", get_timer_vect)?;
    cx.export_function("getTimerPriority", get_timer_priority)?;
    cx.export_function("getTimerMax", get_timer_max)?;
    cx.export_function("setTimerVect", set_timer_vect)?;
    cx.export_function("setTimerPriority", set_timer_priority)?;
    cx.export_function("setTimerMax", set_timer_max)?;
    Ok(())
}
