const Application = require('spectron').Application
const assert = require('assert')
const electronPath = require('electron') // Require Electron from the binaries included in node_modules.
const path = require('path')
fs = require('fs');
var Modele;
describe('Application launch', function () {
  this.timeout(20000)

  before(function () {
    this.app = new Application({
      // Your electron path can be any binary
      // i.e for OSX an example path could be '/Applications/MyApp.app/Contents/MacOS/MyApp'
      // But for the sake of the example we fetch it from our node_modules.
      path: electronPath,

      // Assuming you have the following directory structure

      //  |__ my project
      //     |__ ...
      //     |__ main.js
      //     |__ package.json
      //     |__ index.html
      //     |__ ...
      //     |__ test
      //        |__ spec.js  <- You are here! ~ Well you should be.

      // The following line tells spectron to look and use the main.js file
      // and the package.json located 1 level above.
      args: [path.join(__dirname, '..', 'dist/main.js')]
    })

    //many webdriverIO (app.client) methods not available on returned value
    return this.app.start().then(async () => {
      this.app.client.waitUntilTextExists('#model').then(() => {
        this.model = JSON.stringify(this.app.client.$("#model"));
        this.temp = this.model.clone();
      })
    })
  })
  // beforeEach(function () {
  //   this.app.client.click 
  // })

  after(function () {
    if (this.app && this.app.isRunning()) {
      return this.app.stop()
    }
  })

  afterEach(function () {
    this.app.client.click('#Reset');
    this.model = JSON.stringify(this.app.client.$("#model"));
  })


  it('shows an initial window', function () {
    console.log(this.model);
    return this.app.client.getWindowCount().then(function (count) {
      assert.equal(count, 2)
      // Please note that getWindowCount() will return 2 if `dev tools` are opened.
      // assert.equal(count, 2)
    })

  })

  it('Reset button', async function () {
    
    this.temp = JSON.stringify(this.app.client.$("#model"));
    this.app.client.click('#Reset');
    this.model = JSON.stringify(this.app.client.$("#model"));
    return assert.equal(this.temp, this.model)

  })

  // it('shows Buttons', function () {

  //   return this.app.client.getWindowCount().then(function (count) {
  //     assert.equal(count, 2)
  //     // Please note that getWindowCount() will return 2 if `dev tools` are opened.
  //     // assert.equal(count, 2)
  //   })

  // })

  // it('Toolbar Functions', function () {
  //   return this.app.client.getWindowCount().then(function (count) {
  //     assert.equal(count, 2)
  //     // Please note that getWindowCount() will return 2 if `dev tools` are opened.
  //     // assert.equal(count, 2)
  //   })

  // })

  // it('Add Column', function () {
  //   return this.app.client.getWindowCount().then(function (count) {
  //     assert.equal(count, 2)
  //     // Please note that getWindowCount() will return 2 if `dev tools` are opened.
  //     // assert.equal(count, 2)
  //   })

  // })

  // it('Add Row', function () {
  //   return this.app.client.getWindowCount().then(function (count) {
  //     assert.equal(count, 2)
  //     // Please note that getWindowCount() will return 2 if `dev tools` are opened.
  //     // assert.equal(count, 2)
  //   })

  // })

  // it('Delete Column', function () {
  //   return this.app.client.getWindowCount().then(function (count) {
  //     assert.equal(count, 2)
  //     // Please note that getWindowCount() will return 2 if `dev tools` are opened.
  //     // assert.equal(count, 2)
  //   })

  // })

  // it('Delete Row', function () {
  //   return this.app.client.getWindowCount().then(function (count) {
  //     assert.equal(count, 2)
  //     // Please note that getWindowCount() will return 2 if `dev tools` are opened.
  //     // assert.equal(count, 2)
  //   })

  // })

  // it('ADD -> Delete Row / Column', function () {
  //   return this.app.client.getWindowCount().then(function (count) {
  //     assert.equal(count, 2)
  //     // Please note that getWindowCount() will return 2 if `dev tools` are opened.
  //     // assert.equal(count, 2)
  //   })

  // })

  // it('ADD -> Delete Row & Column', function () {
  //   return this.app.client.getWindowCount().then(function (count) {
  //     assert.equal(count, 2)
  //     // Please note that getWindowCount() will return 2 if `dev tools` are opened.
  //     // assert.equal(count, 2)
  //   })

  // })

  // it('Zoom In & Out', function () {
  //   return this.app.client.getWindowCount().then(function (count) {
  //     assert.equal(count, 2)
  //     // Please note that getWindowCount() will return 2 if `dev tools` are opened.
  //     // assert.equal(count, 2)
  //   })

  // })

  // it('Zoom Reset', function () {
  //   return this.app.client.getWindowCount().then(function (count) {
  //     assert.equal(count, 2)
  //     // Please note that getWindowCount() will return 2 if `dev tools` are opened.
  //     // assert.equal(count, 2)
  //   })

  // })

  // it('Test Values', function () {
  //   return this.app.client.once('ready-to-show', function () {
  //     this.app.getText("model").then(function (text) {
  //       console.log(text);
  //       assert.equal(2, 2)
  //       // Please note that getWindowCount() will return 2 if `dev tools` are opened.
  //       // assert.equal(count, 2)
  //     })
  //   })
  // })

})