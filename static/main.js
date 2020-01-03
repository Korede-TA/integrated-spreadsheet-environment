const { app, protocol, BrowserWindow, remote } = require('electron');
const { readFile } = require("fs");
const { extname } = require("path");
const { URL } = require("url");

let win;

// buffer protocol for serving build artifacts & assets
function createProtocol (scheme, normalize = true) {
	protocol.registerBufferProtocol(scheme,
		(request, respond) => {
			let pathName = new URL(request.url).pathname;
			pathName = decodeURI(pathName); // Needed in case URL contains spaces
			readFile(app.getAppPath() + "/" + pathName, (error, data) => {
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
			nodeIntegration: true,
    },
    titleBarStyle: 'hiddenInset',
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
