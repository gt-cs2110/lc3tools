declare module "lc3-backend" {
    /**
     * Initializes any properties for the LC3 backend.
     */
    export function Init(): void;

    /**
     * Takes a `.bin` file and creates and exports a
     * `.obj` file out of it.
     * @param fp The filepath of the `.bin` file
     */
    export function ConvertBin(fp: string): void;
    
    /**
     * Takes a `.asm` file and creates and exports a
     * `.obj` file out of it.
     * @param fp The filepath of the `.asm` file
     * @throws if assembling fails
     */
    export function Assemble(fp: string): void;
    
    /**
     * Gets the symbol table, mapping each memory address to a label.
     */
    export function GetCurrSymTable(): {[addr: number]: string};
    
    /**
     * Sets the enable liberal ASM configuration.
     * @param status the status to set the configuration to
     */
    export function SetEnableLiberalAsm(status: boolean): boolean;
    
    /**
     * Loads an object file into the simulator,
     * as well as clearing any state from the previous run.
     * 
     * @param fp The `.obj` file to load to the simulator
     */
    export function LoadObjectFile(fp: string): void;
    
    /**
     * Restarts the simulator.
     * 
     * This reinitializes the state of the simulator.
     */
    export function RestartMachine(): void;

    /**
     * Reinitializes the simulator, 
     * wiping the object file and zeroing the state.
     */
    export function ReinitializeMachine(): void;

    /**
     * Randomizes the simulator memory,
     * wiping the object file and randomizing the state.
     */
    export function RandomizeMachine(): void;

    /**
     * Runs the program asynchronously.
     * @param cb Callback to execute in case of errors.
     */
    export function Run(cb: (err: any) => void): void;

    /**
     * Runs the program asynchronously, until a HALT statement is reached.
     * @param cb Callback to execute in case of errors.
     */
    export function RunUntilHalt(cb: (err: any) => void): void;

    /**
     * Steps into the next instruction.
     * @param cb Callback to execute in case of errors.
     */
    export function StepIn(cb: (err: any) => void): void;
    
    /**
     * Step out of this frame.
     * @param cb Callback to execute in case of errors.
     */
    export function StepOut(cb: (err: any) => void): void;
    
    /**
     * Steps over to the next instruction, running entire subroutines if needed.
     * @param cb Callback to execute in case of errors.
     */
    export function StepOver(cb: (err: any) => void): void;
    
    /**
     * Pause the execution of the program.
     */
    export function Pause(): void;
    
    /**
     * Gets the register value from the simulator.
     * @param regName The name of the register. This is one of:
     * `r0`-`r7`, `pc`, `psr`, or `mcr`.
     */
    export function GetRegValue(regName: string): number;
    
    /**
     * Sets the register value to the simulator.
     * @param regName The name of the register. This is one of:
     * `r0`-`r7`, `pc`, `psr`, or `mcr`.
     * @param value Value to set the register to.
     */
    export function SetRegValue(regName: string, value: number): void;
    
    /**
     * Gets the memory value from the simulator.
     * @param addr The memory location to get from.
     */
    export function GetMemValue(addr: number): number;

    /**
     * Sets the memory value to the simulator.
     * @param addr The memory location to write to.
     * @param value Value to set the register to.
     */
    export function SetMemValue(addr: number, value: number): void;

    /**
     * Gets the memory line at this value (the text associated with that line).
     * @param addr The memory location to read the line of.
     */
    export function GetMemLine(addr: number): string;

    /**
     * Sets the memory line at this value (the text associated with that line).
     * @param addr The memory location to write the line of.
     * @param line The line to write
     */
    export function SetMemLine(addr: number, line: string): void;

    /**
     * Sets the ignore privilege configuration.
     * @param status the status to set the configuration to.
     */
    export function SetIgnorePrivilege(status: boolean): boolean;

    /**
     * Clears the simulator console input.
     */
    export function ClearInput(): void;

    /**
     * Adds a character to the console input.
     * @param char The character to add. This should be ONE CHARACTER.
     */
    export function AddInput(char: string): void;

    /**
     * Gets the input from the console (for both the editor or simulator)
     * and clears the internal buffer.
     */
    export function GetAndClearOutput(): string;
    /**
     * Clears the internal buffer for the console 
     * (for both the editor or simulator).
     */
    export function ClearOutput(): void;

    /**
     * Sets a breakpoint at the given memory address.
     * @param addr The memory address to add a breakpoint to.
     */
    export function SetBreakpoint(addr: number): void;
    /**
     * Removes a breakpoint from the given memory address.
     * @param addr The memory address to remove the breakpoint of.
     */
    export function RemoveBreakpoint(addr: number): void;
    /**
     * I have no idea what this does and its output is literally unused
     * in the simulator????
     */
    export function GetInstExecCount(): number;
    /**
     * Checks if a breakpoint was tripped.
     */
    export function DidHitBreakpoint(): boolean;
}