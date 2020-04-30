const Application = require('spectron').Application
const assert = require('assert')
const electronPath = require('electron') // Require Electron from the binaries included in node_modules.
const path = require('path')
fs = require('fs');

// model test helper
Object.prototype.equals = function (object2) {
  //For the first loop, we only check for types
  for (propName in this) {
    //Check for inherited methods and properties - like .equals itself
    //https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/hasOwnProperty
    //Return false if the return value is different
    if (this.hasOwnProperty(propName) != object2.hasOwnProperty(propName)) {
      return false;
    }
    //Check instance type
    else if (typeof this[propName] != typeof object2[propName]) {
      //Different types => not equal
      return false;
    }
  }
  //Now a deeper check using other objects property names
  for (propName in object2) {
    //We must check instances anyway, there may be a property that only exists in object2
    //I wonder, if remembering the checked values from the first loop would be faster or not 
    if (this.hasOwnProperty(propName) != object2.hasOwnProperty(propName)) {
      return false;
    } else if (typeof this[propName] != typeof object2[propName]) {
      return false;
    }
    //If the property is inherited, do not check any more (it must be equa if both objects inherit it)
    if (!this.hasOwnProperty(propName))
      continue;

    //Now the detail check and recursion

    //This returns the script back to the array comparing
    /**REQUIRES Array.equals**/
    if (this[propName] instanceof Array && object2[propName] instanceof Array) {
      // recurse into the nested arrays
      if (!this[propName].equals(object2[propName]))
        return false;
    } else if (this[propName] instanceof Object && object2[propName] instanceof Object) {
      // recurse into another objects
      //console.log("Recursing to compare ", this[propName],"with",object2[propName], " both named \""+propName+"\"");
      if (!this[propName].equals(object2[propName]))
        return false;
    }
    //Normal value comparison for strings and numbers
    else if (this[propName] != object2[propName]) {
      return false;
    }
  }
  //If everything passed, let's say YES
  return true;
}

describe('Application launch', function () {
  this.timeout(5000)
  
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

    // many webdriverIO (app.client) methods not available on returned value
    return this.app.start();
  })


  after(function () {
    if (this.app && this.app.isRunning()) {
      return this.app.stop()
    }
  })

  // Will change before final version
  afterEach(async function () {
    await this.app.client.pause(500)
    await this.app.client.click('#Reset');
    await this.app.client.waitForExist('#integration-test-model-dump');
  })


  it('shows an initial window', async function () {
    //  waits for window to fully load
    await this.app.client.waitForExist('#integration-test-model-dump');
    this.app.client.getWindowCount().then(function (count) {
      return assert.equal(count, 1)
    })

  })

  it('Reset button', async function () {
    var temp = JSON.parse(await this.app.client.$("#integration-test-model-dump").getText()).grammars;
    await this.app.client.click('#Reset');
    var model = JSON.parse(await this.app.client.$("#integration-test-model-dump").getText()).grammars;
    return assert.equal(true, model.equals(temp))
    //Test Reset Button to ensure the Model is reset to the most basic format

  })

  it('shows Buttons', async function () {
    var temp = await this.app.client.$$("button");
    return assert.equal(temp.length, 19);
    // Test the number of buttons loaded to ensure they are all present
  })

// Inserts and deletes check the grammars and DOM after change
  it('insert Row', async function () {
    this.app.client.click('#InsertRow');
    var el = await this.app.client.$('#integration-test-model-dump').getText();
    var temp = JSON.parse(el).grammars.root.kind;
    if (await this.app.client.$('#cell-root-A4').isExisting() && await this.app.client.$('#cell-root-B4').isExisting() ){
      assert.equal(true, temp.sort().equals([
        [1, 1],
        [1, 2],
        [2, 1],
        [2, 2],
        [3, 1],
        [3, 2],
        [4, 1],
        [4, 2]
      ]));
    }else{
      assert.equal(false, true, "elements not present " + await this.app.client.$('#cell-root-A4').isExisting());
    }
  })

  it('insert Column', async function () {
    this.app.client.click('#InsertCol');
    var el = await this.app.client.$('#integration-test-model-dump').getText()
    var temp = JSON.parse(el).grammars.root.kind;
    if (await this.app.client.$('#cell-root-C1').isExisting() && await this.app.client.$('#cell-root-C2').isExisting()){
      assert.equal(true, temp.sort().equals([
        [1, 1],
        [1, 2],
        [1, 3],
        [2, 1],
        [2, 2],
        [2, 3],
        [3, 1],
        [3, 2],
        [3, 3]
      ]));
    }else{
      assert.equal(false, true, "elements not present ", await this.app.client.$('#cell-root-C1'));
    }
  })

  it('Delete Row', async function () {
    this.app.client.click('#DeleteRow');
    var el = await this.app.client.$('#integration-test-model-dump').getText()
    var temp = JSON.parse(el).grammars.root.kind;
    if (!await this.app.client.$('#cell-root-A3').value && !await this.app.client.$('#cell-root-B3').value){
      assert.equal(true, temp.sort().equals([
        [1, 1],
        [1, 2],
        [2, 1],
        [2, 2]
      ]));
    }else{
      assert.equal(false, true, "elements not present");
    }
  })

  it('Delete Column', async function () {
    this.app.client.click('#DeleteCol');
    var el = await this.app.client.$('#integration-test-model-dump').getText()
    var temp = JSON.parse(el).grammars.root.kind;
    if (!await this.app.client.$('#cell-root-B1').value && !await this.app.client.$('#cell-root-B2').value){
      return assert.equal(true, temp.sort().equals([
        [1, 1],
        [2, 1],
        [3, 1]
      ]));
    }else{
      return assert.equal(false, true, "elements not present");
    }
  })

  // Longest combinaison, Test DOM for all zooms 
  it('Zoom In -> Out -> Out -> Reset Zoom', async function () {
    this.app.client.click('#ZoomIn');
    var zoom = await this.app.client.$('#grammars').getCssProperty("zoom");
    assert.equal(1.1, zoom.value);
    this.app.client.click('#ZoomOut');
    this.app.client.click('#ZoomOut');
    zoom = await this.app.client.$('#grammars').getCssProperty("zoom");
    assert.equal(0.9, zoom.value);
    this.app.client.click('#ZoomReset');
    zoom = await this.app.client.$('#grammars').getCssProperty("zoom");
    return assert.equal(1, zoom.value);
  })

  // test Mergure of A1 and B1 DOMwise and grammar wise
  it('Merge', async function () {

    await this.app.client.click('#cell-root-A1');
    await this.app.client.keys('Shift');
    await this.app.client.click('#cell-root-B1');
    this.app.client.keys('Null');
    await this.app.client.click('#Merge');

    var grid = await this.app.client.$('#cell-root-B1').getCssProperty("grid-column");
    return assert.equal("1 / span 2", grid.value);
  })

  // Test Nesting a grid within A1 DOMwise and grammar wise
  it('Nesting Grid', async function () {
    await this.app.client.click('#cell-root-A1');
    this.app.webContents.executeJavaScript('document.getElementById("nest").click();')
    //await this.app.client.click('#nest');
    var temp = JSON.parse(await this.app.client.$('#integration-test-model-dump').getText()).grammars['root-A1'].kind;
    // var grid = await this.app.client.$('#cell-root-B1').getCssProperty("grid-template-areas");
    // console.log(grid);
    if (await this.app.client.$('#cell-root-A1-A1').isExisting() ){
      return assert.equal(true, temp.sort().equals([
        [1, 1],
        [1, 2],
        [1, 3],
        [2, 1],
        [2, 2],
        [2, 3],
        [3, 1],
        [3, 2],
        [3, 3]
      ]));
    }else{
      assert.equal(false, true, "nested elements not present ");
    }
  })

})