use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

use lc3_ensemble::sim::{SimFlags, WordCreateStrategy};
use lc3_ensemble::sim::Simulator;

#[derive(Debug)]
pub(crate) enum SimAccessError {
    NotAvailable,
    Poisoned
}
impl std::fmt::Display for SimAccessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimAccessError::NotAvailable => f.write_str("simulator is currently in the process of running; the state cannot be read or written to while it is running"),
            SimAccessError::Poisoned => f.write_str("simulator was destroyed in the process of running; please reload object files"),
        }
    }
}
pub(crate) enum SimController {
    Idle(Simulator),
    Running {
        mcr: Arc<AtomicBool>,
        handle: mpsc::Receiver<Simulator>,
    },
    Poison
}
impl SimController {
    /// Creates a new idle simulator state.
    pub(crate) fn new(zeroed: bool) -> Self {
        let word_create_strat = match zeroed {
            false => WordCreateStrategy::Unseeded,
            true  => WordCreateStrategy::Known { value: 0 },
        };
        let sim = Simulator::new(SimFlags {
            word_create_strat,
            ..Default::default()
        });

        SimController::Idle(sim)
    }

    /// Attempts to reacquire the simulator state (returning it to idle)
    /// if the simulator is "running" but its process already completed.
    fn try_reacquire(&mut self) {
        use std::sync::mpsc::TryRecvError;

        if let SimController::Running { handle, .. } = self {
            match handle.try_recv() {
                Ok(sim) => *self = SimController::Idle(sim),
                Err(TryRecvError::Empty) => {},
                Err(TryRecvError::Disconnected) => *self = SimController::Poison,
            }
        }
    }

    /// Blocks the thread until the simulator is idle.
    /// 
    /// If SimState is poisoned, then this will raise a Poisoned error.
    fn join(&mut self) -> Result<(), SimAccessError> {
        if let SimController::Running { handle, .. } = self {
            match handle.recv() {
                Ok(sim) => *self = SimController::Idle(sim),
                Err(_)  => *self = SimController::Poison,
            }
        };

        match self {
            SimController::Idle(_) => Ok(()),
            _ => Err(SimAccessError::Poisoned)
        }
    }

    /// Accesses the simulator if it is currently idle.
    pub(crate) fn simulator(&mut self) -> Result<&mut Simulator, SimAccessError> {
        self.try_reacquire();

        match self {
            SimController::Idle(sim) => Ok(sim),
            SimController::Running { .. } => Err(SimAccessError::NotAvailable),
            SimController::Poison => Err(SimAccessError::Poisoned),
        }
    }

    /// Asynchronously executes function on the simulator, if it is currently idle.
    /// 
    /// This requires two closures:
    /// - One that executes instructions with the simulator
    /// - One that does something with the result of the output
    /// 
    /// If the execution of the executor function causes a panic, 
    /// this will cause the SimState to become poisoned.
    pub(crate) fn execute<T>(&mut self, 
            exec: impl FnOnce(&mut Simulator) -> T + Send + 'static,
            close: impl FnOnce(T) + Send + 'static
        ) -> Result<(), SimAccessError> {
        let _ = self.simulator()?; // assert idle
        let SimController::Idle(mut sim) = std::mem::replace(self, SimController::Poison) else {
            // this is assured by the above
            unreachable!("sim state should have been idle");
        };

        let mcr = Arc::clone(sim.mcr());
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let result = exec(&mut sim);
            tx.send(sim).unwrap();
            close(result);
        });

        *self = SimController::Running { mcr, handle: rx };
        Ok(())
    }

    /// Pauses the simulator if is running.
    pub(crate) fn pause(&mut self) -> Result<(), SimAccessError> {
        if let SimController::Running { mcr, .. } = self {
            mcr.store(false, Ordering::Relaxed);
        }
        self.join()
    }

    /// Resets the state of the simulator,
    /// reseting it back to a randomized machine state (with OS).
    /// 
    /// The sim state is idle after this is called.
    pub(crate) fn reset(&mut self, zeroed: bool, flags: SimFlags) -> &mut Simulator {
        let _ = self.pause();
        
        // preserve breakpoints
        let breakpoints = match self.simulator() {
            Ok(old_sim) => std::mem::take(&mut old_sim.breakpoints),
            Err(_) => {
                eprintln!("could not obtain previous simulator's breakpoints, simulator is poisoned");
                vec![]
            },
        };

        let _ = self.pause(); // ignore result
        if self.simulator().is_err() {
            *self = SimController::new(zeroed);
        }

        let sim = self.simulator()
            .unwrap_or_else(|_| unreachable!("sim controller known to be idle"));
        sim.reset();
        sim.flags = flags;
        sim.breakpoints = breakpoints;
        sim
    }
}