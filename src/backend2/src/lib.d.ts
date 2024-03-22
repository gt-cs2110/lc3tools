declare module "lc3-backend" {
    /**
     * Initializes any properties for the LC3 backend.
     */
    export function init(): void;

    /**
     * Takes a `.bin` file and creates and exports a
     * `.obj` file out of it.
     * @param fp The filepath of the `.bin` file
     */
    export function convertBin(fp: string): void;
    
    /**
     * Takes a `.asm` file and creates and exports a
     * `.obj` file out of it.
     * @param fp The filepath of the `.asm` file
     * @throws if assembling fails
     */
    export function assemble(fp: string): void;
    
    /**
     * Gets the symbol table, mapping each memory address to a label.
     */
    export function getCurrSymTable(): {[addr: number]: string};
    
    /**
     * Sets the enable liberal ASM configuration.
     * @param status the status to set the configuration to
     */
    export function setEnableLiberalAsm(status: boolean): boolean;
    
    /**
     * Loads an object file into the simulator,
     * as well as clearing any state from the previous run.
     * 
     * @param fp The `.obj` file to load to the simulator
     */
    export function loadObjectFile(fp: string): void;
    
    /**
     * Restarts the simulator.
     * 
     * This reinitializes the state of the simulator.
     */
    export function restartMachine(): void;

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
     * Runs the program asynchronously, until a HALT statement is reached.
     * @param cb Callback to execute in case of errors.
     */
    export function runUntilHalt(cb: (err: any) => void): void;

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
     * Sets the memory line at this value (the text associated with that line).
     * @param addr The memory location to write the line of.
     * @param line The line to write
     */
    export function setMemLine(addr: number, line: string): void;

    /**
     * Sets the ignore privilege configuration.
     * @param status the status to set the configuration to.
     */
    export function setIgnorePrivilege(status: boolean): boolean;

    /**
     * Clears the simulator console input.
     */
    export function clearInput(): void;

    /**
     * Adds a character to the console input.
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
     * Sets a breakpoint at the given memory address.
     * @param addr The memory address to add a breakpoint to.
     */
    export function setBreakpoint(addr: number): void;
    /**
     * Removes a breakpoint from the given memory address.
     * @param addr The memory address to remove the breakpoint of.
     */
    export function removeBreakpoint(addr: number): void;
    /**
     * I have no idea what this does and its output is literally unused
     * in the simulator????
     */
    export function getInstExecCount(): number;
    /**
     * Checks if a breakpoint was tripped.
     */
    export function didHitBreakpoint(): boolean;
    /**
     * Checks if the simulator is currently running.
     */
    export function isSimRunning(): boolean;
}