// See the Electron documentation for details on how to use preload scripts:
// https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts

// Do not import any modules here.
// This only imports lc3 because it can't be passed easily
// from the main process.

// If you want to add functionality, use ipcRenderer.invoke (& friends)
// and handle it in main.ts.

import { contextBridge, ipcRenderer } from "electron";
import lc3 from "lc3-backend";

contextBridge.exposeInMainWorld("lc3", lc3);
contextBridge.exposeInMainWorld("autoUpdater", {
    on(cb: (event: Electron.IpcRendererEvent, msg: any, progress: any) => void) {
        ipcRenderer.on("auto_updater", cb)
    },
    send(s: string) {
        ipcRenderer.send("auto_updater", s)
    }
})
contextBridge.exposeInMainWorld("dialog", {
    async showModal(type: string, config: any): Promise<any> {
        return ipcRenderer.invoke("show_modal", type, config);
    }
})
contextBridge.exposeInMainWorld("storage", {
    get(k: string): any {
        return ipcRenderer.sendSync("config_get", k);
    },
    set(k: string, v: any): void {
        return ipcRenderer.sendSync("config_set", k, v);
    },
    getAll(): object {
        return ipcRenderer.sendSync("config_get_all");
    },
    setAll(data: object): void {
        return ipcRenderer.sendSync("config_set_all", data);
    }
});