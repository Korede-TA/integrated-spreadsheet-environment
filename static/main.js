const { app, protocol, BrowserWindow, remote } = require('electron');
const { readFile } = require("fs");
const { extname } = require("path");
const { URL } = require("url");

let win;

let driverMiscFiles = {};

// buffer protocol for serving build artifacts & assets
function createProtocol (scheme, normalize = true) {
	protocol.registerBufferProtocol(scheme,
		(request, respond) => {
			let pathName = new URL(request.url).pathname;
			pathName = decodeURI(pathName); // Needed in case URL contains spaces

      function getMimeType(pathName) {
				let extension = extname(pathName).toLowerCase();
				let mimeType = "";
        // enforce mime types
				if (extension === ".js") {
					mimeType = "text/javascript";
				} else if (extension === ".html") {
					mimeType = "text/html";
				} else if (extension === ".css") {
					mimeType = "text/css";
				} else if (extension === ".svg" || extension ===
					".svgz") {
					mimeType = "image/svg+xml";
				} else if (extension === ".json") {
					mimeType = "application/json";
				} else if (extension === ".wasm") {
					mimeType = "application/wasm";
				}
        return mimeType;
      }

      // IF: the file is being requested is among the stored "misc files" for any driver return the file data stored
      if (!!driverMiscFiles[pathName]) {
        respond({
          mimeType: getMimeType(pathName),
          data: driverMiscFiles[pathName].content,
        });
      }

      // OTHERWISE: get the full filepath and read the file from the filesystem.
			readFile(app.getAppPath() + "/" + pathName, (error, data) => {
				respond({
          mimeType: getMimeType(pathName),
					data
				});
			});
		});
}

// IPC: Communication between Electron main.js and Rust src/lib.rs
const { ipcMain } = require('electron');
ipcMain.on('upload-driver-misc-file', (event, args) => {
  console.log(args);

  driverMiscFiles[args[0]] = args[1];

  // respond with success or failure (true/false)
  event.returnValue = true;
});


// standard scheme must be registered before the app is ready
// https://gist.github.com/dbkr/e898624be6d53590ebf494521d868fec
protocol.registerSchemesAsPrivileged([{
    scheme: 'app',
    privileges: { standard: true, secure: true, supportFetchAPI: true },
}]);

function createWindow () {
  win = new BrowserWindow({
    webPreferences: {
			nodeIntegration: true,
    },
  });
  win.loadURL(`file://${__dirname}/index.html`);
  win.webContents.openDevTools(); // TODO: only do this in development mode
  win.once('ready-to-show', () => {
    win.show();
    win.maximize();
  });
  win.on('closed', () => {
    win = null; // dereference window object
  });
}

app.on('ready', () => {
  createProtocol("app");
  createWindow();
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin' /* macOS */ ) {
    app.quit();
  }
});

app.on('activate', () => {
  if (win === null) {
    createWindow();
  }
});
