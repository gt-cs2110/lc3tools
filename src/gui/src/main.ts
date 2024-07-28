import { app, BrowserWindow, dialog, ipcMain, screen } from 'electron';
import path from 'path';
import { AutoUpdaterSendType, showModal } from './types/renderer';

// Handle creating/removing shortcuts on Windows when installing/uninstalling.
if (require('electron-squirrel-startup')) {
  app.quit();
}

const createWindow = () => {
  // Create the browser window.
  let { width, height } = screen.getPrimaryDisplay().size;
  const mainWindow = new BrowserWindow({
    width, height,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
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

  // Open the DevTools.
  mainWindow.webContents.openDevTools();
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

// In this file you can include the rest of your app's specific main process
// code. You can also put them in separate files and import them here.
ipcMain.on("auto_updater", (e, text: AutoUpdaterSendType) => {
  if (text == "update_confirmed") {
    throw new Error("todo");
  } else {
    let _exhaustiveCheck: never = text;
  }
});

ipcMain.handle("show_modal", (e, kind, config) => {
    // Note: If new parameters are accepted into this invocation,
    // the compiler will not indicate so.

    // As such, they have to be added here to be accepted.
    if (kind === "save") {
      return dialog.showSaveDialog(config);
    } else if (kind === "open") {
      return dialog.showOpenDialog(config);
    }
})