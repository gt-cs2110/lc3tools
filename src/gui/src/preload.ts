// See the Electron documentation for details on how to use preload scripts:
// https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts

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