const path = require('path');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin');

const distPath = path.resolve(__dirname, "dist");

module.exports = {
  target: 'electron-renderer',
  entry: {
    app: `${distPath}/bootstrap.js`,
    background: `${distPath}/main.js`,
  },
  output: {
    path: distPath,
    filename: "[name].js",
    webassemblyModuleFilename: "integrated_spreadsheet_environment.wasm",
  },
  module: {
    rules: [{
      test: /\.wasm$/,
      include: [/node_modules/],
      use: [ { loader: 'wasm-loader'} ]
    }]
  },
  // entry: { gui: `${distPath}/integrated_spreadsheet_environment.js` },
  plugins: [
    new CopyWebpackPlugin([
      { from: './static', to: distPath }
    ]),
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "."),
      extraArgs: "--no-typescript",
    }),
  ]
};
