// This file is for declaring types established in preload.ts.
export type LC3Backend = typeof import("lc3-backend");
export type AutoUpdaterSendType = "update_confirmed";
export type AutoUpdaterBindings = {
    on(cb: (msg: any, progress: any) => void): void;
    send(s: AutoUpdaterSendType): void;
};

export async function showModal(type: "save", config: Electron.SaveDialogOptions): Promise<Electron.SaveDialogReturnValue>;
export async function showModal(type: "open", config: Electron.OpenDialogOptions): Promise<Electron.OpenDialogReturnValue>;
export async function showModal(type: "box", config: Electron.MessageBoxOptions): Promise<Electron.MessageBoxReturnValue>;
export async function showModal(type: "menu", config: string[]): Promise<number>;
export type DialogBindings = {
    showModal: typeof showModal
}

export type StorageBindings = {
    get(k: string): any;
    set(k: string, v: any): void;
    getAll(): object;
    setAll(data: object): void;
}

export type FSBindings = {
    read(fp: string): Promise<string>;
    write(fp: string, content: string): Promise<void>;
    exists(fp: string): boolean;
    basename(fp: string): string;
    getPath(f: File): string;
}

export type API = {
    lc3: LC3Backend,
    autoUpdater: AutoUpdaterBindings,
    dialog: DialogBindings,
    storage: StorageBindings,
    fs: FSBindings
};

export type Handler<F> = (e: Electron.IpcMainInvokeEvent, ...args: Parameters<F>) => ReturnType<F> | Awaited<ReturnType<F>>;
export type SyncHandler<F> = (e: Omit<Electron.IpcMainEvent, "returnValue"> & { returnValue: ReturnType<F> }, ...args: Parameters<F>) => void;

// Note: This declares the api object into the global scope.
// However, window.api is NOT accessible in the main process (i.e., main.ts).
// It can only be used in the files belonging to the renderer process (i.e., *.vue and renderer.ts).
//
// The exact implementation details of window.api are found in preload.ts.
declare global {
    interface Window {
        api: API;
    }
}
export default API;