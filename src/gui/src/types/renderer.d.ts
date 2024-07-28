// This file is for declaring types established in preload.ts.
export type LC3Backend = typeof import("lc3-backend");
export type AutoUpdaterSendType = "update_confirmed";
export type AutoUpdaterBindings = {
    on(cb: (event: Electron.IpcRendererEvent, msg: any, progress: any) => void): void;
    send(s: string): void;
};

export async function showModal(type: "save", config: Electron.SaveDialogOptions): Promise<Electron.SaveDialogReturnValue>;
export async function showModal(type: "open", config: Electron.OpenDialogOptions): Promise<Electron.OpenDialogReturnValue>;

export type Dialog = {
    showModal: typeof showModal
}

export type Storage = {
    get(k: string): any;
    set(k: string, v: any): void;
    getAll(): object;
    setAll(data: object): void;
}