// See the Electron documentation for details on how to use preload scripts:
// https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts

// Avoid importing any modules here.
// The current imports are:
// - lc3: Cannot be passed easily from the main process, so needs to be imported here
// - webUtils: webUtils only exists on renderer

// If you want to add functionality, consider using ipcRenderer.invoke (& friends)
// and handle it in main.ts.

import { contextBridge, ipcRenderer, webUtils } from "electron";
import lc3 from "lc3-backend";

contextBridge.exposeInMainWorld("api", {
    lc3,
    dialog: {
        async showModal(type: string, config: any): Promise<any> {
            return ipcRenderer.invoke("show_modal", type, config);
        }
    },
    storage: {
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
    },
    fs: {
        async read(fp: string): Promise<string> {
            return ipcRenderer.invoke("fs_read", fp);
        },
        async write(fp: string, content: string): Promise<void> {
            return ipcRenderer.invoke("fs_write", fp, content);
        },
        exists(fp: string): boolean {
            return ipcRenderer.sendSync("fs_exists", fp);
        },
        basename(fp: string): string {
            return ipcRenderer.sendSync("fs_path_basename", fp);
        },
        getPath(file: File): string {
            return webUtils.getPathForFile(file);
        }
    }
});