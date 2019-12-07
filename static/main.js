const { app, protocol, BrowserWindow, remote } = require('electron');
const { readFile } = require("fs");
const { extname } = require("path");
const { URL } = require("url");

let win; // declare global reference to window object

// set up buffer protocol
function createProtocol (scheme, normalize = true) {
	protocol.registerBufferProtocol(scheme,
		(request, respond) => {
			let pathName = new URL(request.url).pathname;

			// Needed in case URL contains spaces
			pathName = decodeURI(pathName);

      console.log(pathName);

			readFile(app.getAppPath() + "/" + pathName, function (error, data) {
				let extension = extname(pathName).toLowerCase();
				let mimeType = "";
        // Enforce mime types
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
				respond({
					mimeType,
					data
				});
			});
		});
}

// standard scheme must be registered before the app is ready
// https://gist.github.com/dbkr/e898624be6d53590ebf494521d868fec
protocol.registerSchemesAsPrivileged([{
    scheme: 'app',
    privileges: { standard: true, secure: true, supportFetchAPI: true },
}]);


function createWindow () {
  win = new BrowserWindow({
    webPreferences: {
      // preload: `${__dirname}/preload.js`,
			nodeIntegration: true,
			// contextIsolation: true
    }
  });

  win.loadURL(`file://${__dirname}/index.html`);

  win.webContents.openDevTools(); // TODO: only do this in development mode

  win.maximize();
  win.show();

  win.on('closed', () => {
    win = null; // dereference window object
  });
}

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.on('ready', function() {
  createProtocol("app");
  createWindow();
});

// Quit when all windows are closed
app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    // on macOS, the app sometimes stays open while
    // windows are open
    app.quit();
  }
});

app.on('activate', () => {
  // on macOS it's common to re-create a window in the app when the
  // dock icon is clicked and there are no other windows open.
  if (win === null) {
    createWindow();
  }
});
