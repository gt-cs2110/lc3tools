import { app, BrowserWindow, dialog, ipcMain, Menu, screen } from 'electron';
import Store from 'electron-store';
import fs from 'fs';
import path from 'path';
import { API, Handler, SyncHandler } from './api';
import electronSquirrelStartupFailure from 'electron-squirrel-startup';
import { updateElectronApp } from 'update-electron-app';

// Handle creating/removing shortcuts on Windows when installing/uninstalling.
if (electronSquirrelStartupFailure) {
  app.quit();
}
// Auto-updater.
updateElectronApp();

// Only allow devTools in development mode:
const enableDevTools = process.env.NODE_ENV === "development";
// const enableDevTools = true;

const createWindow = () => {
  // Create the browser window.
  const { width, height } = screen.getPrimaryDisplay().size;
  const mainWindow = new BrowserWindow({
    width, height,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      devTools: enableDevTools,

      // Needed to import lc3-backend in preload.ts
      // 
      // For most apps, sandboxing is the best choice. 
      // In certain use cases that are incompatible with the sandbox 
      // (for instance, when using native node modules in the renderer), 
      // it is possible to disable the sandbox for specific processes. 
      // This comes with security risks, especially if any untrusted code 
      // or content is present in the unsandboxed process.
      // https://www.electronjs.org/docs/latest/tutorial/sandbox
      //
      // I'm gonna hope there's not
      sandbox: false
    },
  });

  // and load the index.html of the app.
  if (MAIN_WINDOW_VITE_DEV_SERVER_URL) {
    mainWindow.loadURL(MAIN_WINDOW_VITE_DEV_SERVER_URL);
  } else {
    mainWindow.loadFile(path.join(__dirname, `../renderer/${MAIN_WINDOW_VITE_NAME}/index.html`));
  }

  if (enableDevTools) {
    // Open the DevTools.
    mainWindow.webContents.openDevTools();
  }

  // Set title
  mainWindow.webContents.on("did-finish-load", () => {
    mainWindow.setTitle("LC3Tools v" + app.getVersion());
  })

  Menu.setApplicationMenu(createMenu());
};

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.on('ready', createWindow);

// Quit when all windows are closed, except on macOS. There, it's common
// for applications and their menu bar to stay active until the user quits
// explicitly with Cmd + Q.
app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('activate', () => {
  // On OS X it's common to re-create a window in the app when the
  // dock icon is clicked and there are no other windows open.
  if (BrowserWindow.getAllWindows().length === 0) {
    createWindow();
  }
});

const createMenu = () => {
  // Mostly copied from: 
  // https://github.com/electron/electron/blob/1c3a5ba5d17c18cbc1fc096d2a05fc24f2b2ddee/lib/browser/default-menu.ts#L12-L58
  const isMac = process.platform === 'darwin';
  const macAppMenu: Electron.MenuItemConstructorOptions = { role: 'appMenu' };

  const template: Electron.MenuItemConstructorOptions[] = [
    ...(isMac ? [macAppMenu] : []),
    {
      role: 'fileMenu',
      submenu: [
        // Creates a new window!
        {
          label: "New Window",
          accelerator: "CommandOrControl+N",
          click: () => createWindow()
        },
        isMac ? { role: 'close' } : { role: 'quit' }
      ]
    },
    { role: 'editMenu' },
    { 
      role: 'viewMenu',
      submenu: [
        ...(enableDevTools ? [
          { role: "toggleDevTools" },
          { type: "separator" },
        ] satisfies Electron.MenuItemConstructorOptions[] : []),
        { role: "resetZoom" },
        { role: "zoomIn" },
        { role: "zoomOut" },
        { type: "separator" },
        { role: "togglefullscreen" },
      ]
    },
    { role: 'windowMenu' }
  ];

  return Menu.buildFromTemplate(template);
}
// In this file you can include the rest of your app's specific main process
// code. You can also put them in separate files and import them here.

// modals
ipcMain.handle("show_modal", (e, kind, config) => {
    // Note: If new parameters are accepted into this invocation,
    // the compiler will not indicate so.

    // As such, they have to be added here to be accepted.
    if (kind === "save") {
      return dialog.showSaveDialog(config);
    } else if (kind === "open") {
      return dialog.showOpenDialog(config);
    } else if (kind === "box") {
      return dialog.showMessageBox(config);
    } else if (kind === "menu") {
      return new Promise((resolve) => {
        const template = Array.from(config ?? [], (label, i) => ({
          label: label.toString(),
          click: () => resolve(i)
        }));
        const menu = Menu.buildFromTemplate(template);
        menu.popup({window: BrowserWindow.fromWebContents(e.sender)})
      })
    }
});

// config storage
const store = new Store();

ipcMain.on("config_get", ((e, key: string) => {
  e.returnValue = store.get(key);
}) satisfies SyncHandler<API["storage"]["get"]>);

ipcMain.on("config_set", ((e, key: string, val: any) => {
  store.set(key, val);
  e.returnValue = undefined;
}) satisfies SyncHandler<API["storage"]["set"]>);

ipcMain.on("config_get_all", (e => {
  e.returnValue = store.store;
}) satisfies SyncHandler<API["storage"]["getAll"]>);

ipcMain.on("config_set_all", ((e, data: object) => {
  store.set(data);
  e.returnValue = undefined;
}) satisfies SyncHandler<API["storage"]["setAll"]>);

// fs
ipcMain.handle("fs_read", ((e, fp: string) => {
  return fs.readFileSync(fp, "utf-8");
}) satisfies Handler<API["fs"]["read"]>)

ipcMain.handle("fs_write", ((e, fp: string, content: string) => {
  fs.writeFileSync(fp, content);
}) satisfies Handler<API["fs"]["write"]>)

ipcMain.on("fs_exists", ((e, fp: string) => {
  e.returnValue = fs.existsSync(fp);
}) satisfies SyncHandler<API["fs"]["exists"]>)
ipcMain.on("fs_path_basename", ((e, fp) => {
  e.returnValue = path.basename(fp);
}) satisfies SyncHandler<API["fs"]["basename"]>)