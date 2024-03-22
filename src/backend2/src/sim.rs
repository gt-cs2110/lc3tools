use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

use lc3_ensemble::sim::Simulator;

#[derive(Debug)]
pub(crate) enum SimAccessError {
    NotAvailable,
    Poisoned
}

pub(crate) enum SimState {
    Idle(Simulator),
    Running {
        mcr: Arc<AtomicBool>,
        handle: mpsc::Receiver<Simulator>,
    },
    Poison
}
impl SimState {
    /// Creates a new idle simulator state.
    pub(crate) fn new() -> Self {
        SimState::Idle({
            let mut sim = Simulator::new();
            sim.load_os();
            sim
        })
    }

    /// Attempts to reacquire the simulator state (returning it to idle)
    /// if the simulator is "running" but its process already completed.
    fn try_reacquire(&mut self) {
        use std::sync::mpsc::TryRecvError;

        if let SimState::Running { handle, .. } = self {
            match handle.try_recv() {
                Ok(sim) => *self = SimState::Idle(sim),
                Err(TryRecvError::Empty) => {},
                Err(TryRecvError::Disconnected) => *self = SimState::Poison,
            }
        }
    }

    /// Blocks the thread until the simulator is idle.
    /// 
    /// If SimState is poisoned, then this will raise a Poisoned error.
    fn join(&mut self) -> Result<(), SimAccessError> {
        if let SimState::Running { handle, .. } = self {
            match handle.recv() {
                Ok(sim) => *self = SimState::Idle(sim),
                Err(_)  => *self = SimState::Poison,
            }
        };

        match self {
            SimState::Idle(_) => Ok(()),
            _ => Err(SimAccessError::Poisoned)
        }
    }

    /// Accesses the simulator if it is currently idle.
    pub(crate) fn simulator(&mut self) -> Result<&mut Simulator, SimAccessError> {
        self.try_reacquire();

        match self {
            SimState::Idle(sim) => Ok(sim),
            SimState::Running { .. } => Err(SimAccessError::NotAvailable),
            SimState::Poison => Err(SimAccessError::Poisoned),
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
        let SimState::Idle(mut sim) = std::mem::replace(self, SimState::Poison) else {
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

        *self = SimState::Running { mcr, handle: rx };
        Ok(())
    }

    /// Pauses the simulator if is running.
    pub(crate) fn pause(&mut self) -> Result<(), SimAccessError> {
        if let SimState::Running { mcr, .. } = self {
            mcr.store(false, Ordering::Relaxed);
        }
        self.join()
    }

    /// Resets the state of the simulator,
    /// reseting it back to a randomized machine state (with OS).
    /// 
    /// The sim state is idle after this is called.
    pub(crate) fn reset(&mut self) {
        let _ = self.pause();
        *self = SimState::new();
    }
}