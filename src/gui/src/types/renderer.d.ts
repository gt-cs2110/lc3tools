// This file is for declaring types established in preload.ts.
export type LC3Backend = typeof import("lc3-backend");
export type AutoUpdaterSendType = "update_confirmed";
export type AutoUpdater = {
    on: (cb: (event: Electron.IpcRendererEvent, msg: any, progress: any) => void) => void;
    send: (type: AutoUpdaterSendType) => void;
};

export async function showModal(type: "save", config: Electron.SaveDialogOptions): Promise<Electron.SaveDialogReturnValue>;
export async function showModal(type: "open", config: Electron.OpenDialogOptions): Promise<Electron.OpenDialogReturnValue>;

export type Dialog = {
    showModal: typeof showModal
}