declare module "lc3-backend" {
    /**
     * Takes a `.asm` file and creates and exports a
     * `.obj` file out of it.
     * @param fp The filepath of the `.asm` file
     * @throws if assembling fails
     */
    export function assemble(fp: string): void;
    
    /**
     * Takes several `.obj` files and links them.
     * @param fps The filepaths of the `.obj` files
     * @param out The output path where the linked object file should be
     * @throws if linking fails
     */
    export function link(fps: string[], out: string): void;

    /**
     * Gets the symbol table, mapping each memory address to a label.
     */
    export function getCurrSymTable(): {[addr: number]: string};
    
    /**
     * Sets the ignore privilege configuration.
     * @param status the status to set the configuration to.
     */
    export function setIgnorePrivilege(status: boolean): void;

    /**
     * Sets the pause on fatal trap configuration.
     * @param status the status to set the configuration to.
     */
    export function setPauseOnFatalTrap(status: boolean): void;
    
    /**
     * Loads an object file into the simulator,
     * as well as clearing any state from the previous run.
     * 
     * @param fp The `.obj` file to load to the simulator
     */
    export function loadObjectFile(fp: string): void;
    
    /**
     * Reinitializes the simulator, 
     * wiping the object file and zeroing the state.
     */
    export function reinitializeMachine(): void;

    /**
     * Randomizes the simulator memory,
     * wiping the object file and randomizing the state.
     */
    export function randomizeMachine(): void;

    /**
     * Runs the program asynchronously.
     * @param cb Callback to execute in case of errors.
     */
    export function run(cb: (err: any) => void): void;

    /**
     * Steps into the next instruction.
     * @param cb Callback to execute in case of errors.
     */
    export function stepIn(cb: (err: any) => void): void;
    
    /**
     * Step out of this frame.
     * @param cb Callback to execute in case of errors.
     */
    export function stepOut(cb: (err: any) => void): void;
    
    /**
     * Steps over to the next instruction, running entire subroutines if needed.
     * @param cb Callback to execute in case of errors.
     */
    export function stepOver(cb: (err: any) => void): void;
    
    /**
     * Pause the execution of the program.
     */
    export function pause(): void;
    
    /**
     * Gets the register value from the simulator.
     * @param regName The name of the register. This is one of:
     * `r0`-`r7`, `pc`, `psr`, or `mcr`.
     */
    export function getRegValue(regName: string): number;
    
    /**
     * Sets the register value to the simulator.
     * @param regName The name of the register. This is one of:
     * `r0`-`r7`, `pc`, `psr`, or `mcr`.
     * @param value Value to set the register to.
     */
    export function setRegValue(regName: string, value: number): void;
    
    /**
     * Gets the memory value from the simulator.
     * @param addr The memory location to get from.
     */
    export function getMemValue(addr: number): number;

    /**
     * Sets the memory value to the simulator.
     * @param addr The memory location to write to.
     * @param value Value to set the register to.
     */
    export function setMemValue(addr: number, value: number): void;

    /**
     * Gets the memory line at this value (the text associated with that line).
     * @param addr The memory location to read the line of.
     */
    export function getMemLine(addr: number): string;

    /**
     * Accesses the list of memory changes that occurred last execution.
     * 
     * It also wipes the list (making it empty if accessed again through this method)
     * and updates the memory lines in accordance to the memory updates.
     */
    export function takeMemChanges(): number[];

    /**
     * Clears the simulator console input.
     */
    export function clearInput(): void;

    /**
     * Adds a character to the console input.
     * This does nothing unless the simulator is currently running.
     * 
     * @param char The character to add. This should be ONE CHARACTER.
     */
    export function addInput(char: string): void;

    /**
     * Gets the input from the console (for both the editor or simulator)
     * and clears the internal buffer.
     */
    export function getAndClearOutput(): string;
    /**
     * Clears the internal buffer for the console 
     * (for both the editor or simulator).
     */
    export function clearOutput(): void;

    /**
     * Gets the list of breakpoints currently registered in the engine.
     */
    export function getBreakpoints(): number[];
    /**
     * Sets a breakpoint at the given memory address.
     * @param addr The memory address to add a breakpoint to.
     * @return whether inserting the breakpoint was successful
     */
    export function setBreakpoint(addr: number): boolean;
    /**
     * Removes a breakpoint from the given memory address.
     * @param addr The memory address to remove a breakpoint to.
     * @return whether removing the breakpoint was successful
     */
    export function removeBreakpoint(addr: number): boolean;
    /**
     * Checks if a breakpoint was tripped.
     */
    export function didHitBreakpoint(): boolean;
    
    /**
     * Gets the frame number (number of calls deep) from the engine.
     */
    export function getFrameNumber(): number;
    /**
     * Checks if the simulator is currently running.
     */
    export function isSimRunning(): boolean;

    /**
     * Gets the span in source code that corresponds to a given label.
     */
    export function getLabelSourceRange(label: string): [start_lno: number, start_cno: number, end_lno: number, end_cno: number] | undefined;
    /**
     * Gets the span in source code that corresponds to a given memory address.
     */
    export function getAddrSourceRange(addr: number): [start_lno: number, start_cno: number, end_lno: number, end_cno: number] | undefined;

    /**
     * Gets the amount of instructions remaining on the timer device until next interrupt.
     */
    export function getTimerRemaining(): number;
    /**
     * Sets whether the timer is enabled or not.
     * @param status the status
     */
    export function setTimerStatus(status: boolean);
    /**
     * Resets the timer.
     */
    export function resetTimer();
    /**
     * Gets the timer's interrupt vector.
     */
    export function getTimerVect(): number;
    /**
     * Gets the timer's priority. Must be 0-7.
     */
    export function getTimerPriority(): number;
    /**
     * Gets the total time of the timer.
     * 
     * This is  a half-implementation, 
     * because it only gets the minimum possible wait time.
     */
    export function getTimerMax(): number;
    /**
     * Sets the timer's interrupt vector.
     * @param vect the vector
     */
    export function setTimerVect(vect: number);
    /**
     * Sets the timer's priority. Must be 0-7.
     * @param priority the priority
     */
    export function setTimerPriority(priority: number);
    /**
     * Sets the total time of the timer.
     * Every time an interrupt occurs, the timer resets to some number within this range.
     * @param min The minimum value (inclusive)
     * @param max The maximum value (inclusive, defaults to min)
     */
    export function setTimerMax(min: number, max?: number);
}