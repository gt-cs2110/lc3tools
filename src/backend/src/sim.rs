use std::collections::VecDeque;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, MutexGuard, RwLockWriteGuard};
use std::thread::JoinHandle;

use lc3_ensemble::sim::device::{BufferedDisplay, BufferedKeyboard, TimerDevice};
use lc3_ensemble::sim::mem::Word;
use lc3_ensemble::sim::{MemAccessCtx, SimFlags, MCR};
use lc3_ensemble::sim::Simulator;

#[derive(Debug)]
pub(crate) struct NotAvailableError;
impl std::fmt::Display for NotAvailableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Simulator's state cannot be read from or written to while it is running.")
    }
}
impl std::error::Error for NotAvailableError {}

// This is a little hacky, cause we're not using mutexes typically here.
// Note that all methods that access the simulator from SimController require &mut.
//
// That's because if we're not executing anything, we're treating this as a unique Arc
// which can mutably access the simulator.
//
// If we are executing something, the simulator moves to a second thread (so two references to Arc).
// The mutex then allows the executing thread to access the Simulator (while preventing
// the JS-interfacing thread from doing so).
// When the simulator joins, the Arc in the executing thread is dropped and we return to a unique Arc.
pub(crate) struct SimController {
    simulator: Arc<Mutex<Simulator>>,
    exec_join: Option<JoinHandle<()>>,

    flags: SimFlags,
    mcr: MCR,
    input: BufferedKeyboard,
    output: BufferedDisplay,
    timer: Arc<Mutex<TimerDevice>>
}
impl SimController {
    pub(crate) fn new() -> Self {
        let flags = Default::default();
        let mut sim = Simulator::new(flags);

        let mcr = Arc::clone(sim.mcr());
        let input = BufferedKeyboard::default();
        let output = BufferedDisplay::default();
        let timer = Arc::default();

        sim.device_handler.set_keyboard(input.clone());
        sim.device_handler.set_display(output.clone());
        sim.device_handler.add_device(Arc::clone(&timer), &[]).expect("should've been able to add timer device");

        Self { simulator: Arc::new(Mutex::new(sim)), exec_join: None, flags, mcr, input, output, timer }
    }

    /// Updates the simulator flags.
    pub(crate) fn update_flags(&mut self, update: impl FnOnce(&mut SimFlags)) {
        update(&mut self.flags);
        // Update simulator's flags, if possible.
        // If not, just do it later.
        let flags = self.flags;
        if let Ok(sim) = self.simulator() {
            sim.flags = flags;
        }
    }

    // I/O
    pub(crate) fn input_buf(&self) -> RwLockWriteGuard<'_, VecDeque<u8>> {
        self.input.get_buffer().write().unwrap_or_else(|e| e.into_inner())
    }
    pub(crate) fn output_buf(&self) -> RwLockWriteGuard<'_, Vec<u8>> {
        self.output.get_buffer().write().unwrap_or_else(|e| e.into_inner())
    }
    pub(crate) fn timer(&self) -> MutexGuard<'_, TimerDevice> {
        self.timer.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Accesses the simulator if it is idle.
    pub(crate) fn simulator(&mut self) -> Result<&mut Simulator, NotAvailableError> {
        self.simulator.clear_poison();
        
        Arc::get_mut(&mut self.simulator)
            .ok_or(NotAvailableError)?
            .get_mut()
            .map_err(|_| unreachable!("poison should have been cleared"))
    }
    /// Checks if simulator is idle.
    pub(crate) fn is_running(&mut self) -> bool {
        self.simulator().is_err()
    }

    /// Pauses the simulator if is running.
    pub(crate) fn pause(&mut self) -> &mut Simulator {
        self.mcr.store(false, Ordering::Relaxed);
        
        // Thread should join soon.
        if let Some(thread) = self.exec_join.take() {
            thread.join().unwrap();
        }

        self.simulator()
            .unwrap_or_else(|_| unreachable!("simulator has reference in another thread"))
    }

    /// Resets the state of the simulator,
    /// reseting it back to a randomized machine state (with OS).
    /// 
    /// The sim state is idle after this is called.
    pub(crate) fn reset(&mut self) -> &mut Simulator {
        let flags = self.flags;
        
        let sim = self.pause();
        sim.flags = flags;
        sim.reset();
        sim
    }

    /// Asynchronously executes function on the simulator, if it is currently idle.
    /// 
    /// This requires two closures:
    /// - One that executes instructions with the simulator
    /// - One that does something with the result of the output
    pub(crate) fn execute<T>(&mut self, 
        exec: impl FnOnce(&mut Simulator) -> T + Send + 'static,
        close: impl FnOnce(T) + Send + 'static
    ) -> Result<(), NotAvailableError> {
        if self.is_running() { return Err(NotAvailableError) };
        
        let flags = self.flags;
        let sim = Arc::clone(&self.simulator);
        let thread = std::thread::spawn(move || {
            let result = {
                let sim = sim; // move Arc ref here so it drops after mutex frees
                let mut guard = sim.lock().unwrap_or_else(|e| e.into_inner());
                guard.flags = flags;
                exec(&mut guard)
            };
            close(result);
        });
        self.exec_join.replace(thread);

        Ok(())
    }

    pub(crate) fn read_mem(&mut self, addr: u16) -> Result<Word, NotAvailableError> {
        self.simulator()?
            .read_mem(addr, MemAccessCtx::omnipotent())
            .map_err(|_| panic!("omnipotent read resulted in error"))
    }

    pub(crate) fn write_mem(&mut self, addr: u16, word: u16) -> Result<(), NotAvailableError> {
        self.simulator()?
            .write_mem(addr, Word::new_init(word), MemAccessCtx::omnipotent())
            .map_err(|_| panic!("omnipotent write resulted in error"))
    }
}
impl Default for SimController {
    fn default() -> Self {
        Self::new()
    }
}