mod print;

use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use lc3_ensemble::asm::{assemble_debug, ObjectFile};
use lc3_ensemble::ast::reg_consts::{R0, R1, R2, R3, R4, R5, R6, R7};
use lc3_ensemble::parse::parse_ast;
use lc3_ensemble::sim::debug::{Breakpoint, Comparator};
use lc3_ensemble::sim::mem::{MemAccessCtx, Word};
use lc3_ensemble::sim::{SimErr, Simulator};
use neon::prelude::*;
use once_cell::sync::Lazy;
use print::{report_error, report_simple, PrintBuffer};

static PRINT_BUFFER: Mutex<PrintBuffer> = Mutex::new(PrintBuffer::new());
static SIMULATOR_CONTENTS: Lazy<Mutex<SimPageContents>> = Lazy::new(|| Mutex::new(SimPageContents {
    sim_state: SimState::Idle({
        let mut sim = Simulator::new();
        sim.load_os();
        sim
    }),
    obj_file: None
}));

struct SimPageContents {
    sim_state: SimState,
    obj_file: Option<ObjectFile>
}
#[derive(Default)]
enum SimState {
    Idle(Simulator),
    Running {
        mcr: Arc<AtomicBool>,
        handle: JoinHandle<(Simulator, Result<(), SimErr>)>
    },
    #[default]
    Poison
}
impl SimState {
    /// Accesses the simulator if it is currently idle.
    fn simulator(&mut self) -> Result<&mut Simulator, ()> {
        match self {
            SimState::Idle(sim) => Ok(sim),
            SimState::Running { .. } => Err(()),
            SimState::Poison => Err(()),
        }
    }

    /// Asynchronously executes function on the simulator, if it is currently idle.
    fn execute(&mut self, f: impl FnOnce(&mut Simulator) -> Result<(), SimErr> + Send + 'static) -> Result<(), ()> {
        let mut sim = match std::mem::take(self) {
            SimState::Idle(sim) => Ok(sim),
            SimState::Running { .. } => Err(()),
            SimState::Poison => Err(()),
        }?;

        let mcr = Arc::clone(sim.mcr());
        let handle = std::thread::spawn(move || {
            println!("executing function");
            let result = f(&mut sim);
            println!("executing done: {result:?}");
            (sim, result)
        });

        *self = SimState::Running { mcr, handle };
        Ok(())
    }

    /// Runs the simulator if it is currently idle.
    fn run(&mut self) -> Result<(), ()> {
        self.execute(Simulator::run)
    }

    /// Pauses the simulator if is running.
    fn pause(&mut self) -> Result<(), ()> {
        if let SimState::Running { mcr, handle: _ } = self {
            mcr.store(false, Ordering::Relaxed);
        }
        self.join()?;
        Ok(())
    }

    /// Waits for the simulator to be idle again.
    fn join(&mut self) -> Result<(), ()> {
        match self {
            SimState::Idle(_) => Ok(()),
            SimState::Running { mcr: _, handle: _ } => {
                let SimState::Running { mcr: _, handle } = std::mem::take(self) else {
                    unreachable!("already checked to be running");
                };

                let (sim, _) = handle.join().unwrap();
                *self = SimState::Idle(sim);
                Ok(())
            },
            SimState::Poison => Err(()),
        }
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
    // fn () -> Result<Object>
    let obj = cx.empty_object();

    let contents = SIMULATOR_CONTENTS.lock().unwrap();
    let Some(obj_file) = contents.obj_file.as_ref() else { return Ok(obj) };
    let Some(_) = obj_file.symbol_table() else { return Ok(obj) };
    Ok(obj)
}
fn load_object_file(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fp: string) -> Result<()>
    let fp = cx.argument::<JsString>(0)?.value(&mut cx);
    let in_path = AsRef::<Path>::as_ref(&fp);
    
    // should be unreachable cause frontend validates IO
    let bytes = std::fs::read(in_path).unwrap();
    
    let obj = ObjectFile::read_bytes(&bytes).unwrap();
    let mut contents = SIMULATOR_CONTENTS.lock().unwrap();
    contents.sim_state.simulator().unwrap().load_obj_file(&obj);
    contents.obj_file.replace(obj);
    
    Ok(cx.undefined())
}
fn restart_machine(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn () -> Result<()>
    // idk what this does
    *SIMULATOR_CONTENTS.lock().unwrap().sim_state.simulator().unwrap() = Simulator::new();
    Ok(cx.undefined())
}
fn reinitialize_machine(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn () -> Result<()>
    
    // TODO: actually zero out memory properly
    *SIMULATOR_CONTENTS.lock().unwrap().sim_state.simulator().unwrap() = Simulator::new();
    Ok(cx.undefined())
}
fn randomize_machine(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fn(err) -> ()) -> Result<()>
    
    *SIMULATOR_CONTENTS.lock().unwrap().sim_state.simulator().unwrap() = Simulator::new();
    Ok(cx.undefined())
}
fn run(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fn(err) -> ()) -> Result<()>
    let done_cb = cx.argument::<JsFunction>(0)?;
    SIMULATOR_CONTENTS.lock().unwrap().sim_state.run();

    Ok(cx.undefined())
}
fn run_until_halt(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fn(err) -> ()) -> Result<()>
    let done_cb = cx.argument::<JsFunction>(0)?;
    SIMULATOR_CONTENTS.lock().unwrap().sim_state.run();

    Ok(cx.undefined())
}
fn step_in(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fn(err) -> ()) -> Result<()>
    let done_cb = cx.argument::<JsFunction>(0)?;
    SIMULATOR_CONTENTS.lock().unwrap().sim_state.execute(Simulator::step_in);

    Ok(cx.undefined())
}
fn step_out(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fn(err) -> ()) -> Result<()>
    let done_cb = cx.argument::<JsFunction>(0)?;
    SIMULATOR_CONTENTS.lock().unwrap().sim_state.execute(Simulator::step_out);

    Ok(cx.undefined())
}
fn step_over(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn (fn(err) -> ()) -> Result<()>
    let done_cb = cx.argument::<JsFunction>(0)?;
    SIMULATOR_CONTENTS.lock().unwrap().sim_state.execute(Simulator::step_over);

    Ok(cx.undefined())
}
fn pause(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    SIMULATOR_CONTENTS.lock().unwrap().sim_state.pause();
    Ok(cx.undefined())
}
fn get_reg_value(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // fn(reg: String) -> Result<u16>
    // reg here can be R0-7, PC, PSR, MCR
    let reg = cx.argument::<JsString>(0)?.value(&mut cx);

    let mut sim_contents = SIMULATOR_CONTENTS.lock().unwrap();
    let simulator = sim_contents.sim_state.simulator().unwrap();

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
        "psr" => simulator.psr().0,
        "mcr" => {
            let mcr = simulator.mcr();
            if mcr.load(Ordering::Relaxed) { 0x8000 } else { 0x0000 }
        }
        _ => panic!("not defined register")
    };
    std::mem::drop(sim_contents);
    
    Ok(cx.number(value))
}
fn set_reg_value(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // fn(reg: String, value: u16) -> Result<()>
    // reg here can be R0-7, PC, PSR, MCR
    let reg = cx.argument::<JsString>(0)?.value(&mut cx);
    let value = cx.argument::<JsNumber>(1)?.value(&mut cx) as u16;

    let mut sim_contents = SIMULATOR_CONTENTS.lock().unwrap();
    let simulator = sim_contents.sim_state.simulator().unwrap();

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
        "psr" => panic!("cannot set PSR"),
        "mcr" => {
            let mcr = simulator.mcr();
            mcr.store((value as i16) < 0, Ordering::Relaxed);
        }
        _ => panic!("not defined register")
    };
    std::mem::drop(sim_contents);
    
    Ok(cx.number(value))
}
fn get_mem_value(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // fn (addr: u16) -> Result<u16>
    let addr = cx.argument::<JsNumber>(0)?.value(&mut cx) as u16;

    let mut sim_contents = SIMULATOR_CONTENTS.lock().unwrap();
    let simulator = sim_contents.sim_state.simulator().unwrap();

    let value = simulator.mem.get(addr, MemAccessCtx { privileged: true, strict: false, io: simulator.io.as_ref() })
        .unwrap()
        .get();
    Ok(cx.number(value))
}
fn set_mem_value(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // fn (addr: u16, value: u16) -> Result<()>
    let addr  = cx.argument::<JsNumber>(0)?.value(&mut cx) as u16;
    let value = cx.argument::<JsNumber>(1)?.value(&mut cx) as u16;
    
    let mut sim_contents = SIMULATOR_CONTENTS.lock().unwrap();
    let simulator = sim_contents.sim_state.simulator().unwrap();

    simulator.mem.set(addr, Word::new_init(value), MemAccessCtx { privileged: true, strict: false, io: simulator.io.as_ref() })
        .unwrap();
    Ok(cx.number(value))
}
fn get_mem_line(mut cx: FunctionContext) -> JsResult<JsString> {
    // fn(addr: u16) -> Result<String>
    Ok(cx.string("?"))
}
fn set_mem_line(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn(addr: u16, value: String) -> Result<()>
    // TODO: implement
    Ok(cx.undefined())
}
fn clear_input(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn() -> ()
    // TODO: implement
    Ok(cx.undefined())
}

fn add_input(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn(input: string) -> Result<()>
    // string is supposed to be char, though
    // TODO: implement
    Ok(cx.undefined())
}

fn set_breakpoint(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn(addr: u16) -> Result<()>
    let addr = cx.argument::<JsNumber>(0)?.value(&mut cx) as u16;
    SIMULATOR_CONTENTS.lock().unwrap()
        .sim_state
        .simulator()
        .unwrap()
        .breakpoints
        .push(Breakpoint::PC(Comparator::eq(addr)));
    Ok(cx.undefined())
}

fn remove_breakpoint(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // fn(addr: u16) -> Result<()>
    let addr = cx.argument::<JsNumber>(0)?.value(&mut cx) as u16;
    SIMULATOR_CONTENTS.lock().unwrap()
        .sim_state
        .simulator()
        .unwrap()
        .breakpoints
        .retain(|bp| bp != &Breakpoint::PC(Comparator::eq(addr)));
    Ok(cx.undefined())
}

fn get_inst_exec_count(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // fn() -> Result<usize>
    // I have no idea what this does
    Ok(cx.number(0))
}

fn did_hit_breakpoint(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    // fn() -> Result<bool>
    // TODO: implement
    Ok(cx.boolean(false))
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
