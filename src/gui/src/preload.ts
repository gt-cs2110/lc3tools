// See the Electron documentation for details on how to use preload scripts:
// https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts

import { contextBridge } from "electron";
import lc3 from "lc3-backend";

contextBridge.exposeInMainWorld("lc3", lc3);