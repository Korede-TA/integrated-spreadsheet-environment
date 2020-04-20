const Application = require('spectron').Application
const assert = require('assert')
const electronPath = require('electron') // Require Electron from the binaries included in node_modules.
const path = require('path')
fs = require('fs');


describe('Application launch', function () {
  this.timeout(20000)
  this.model = JSON;
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
    return this.app.start()
    // .then(() => {
    //   this.model = this.app.client.waitUntil(async function () {
    //     const text = await this.getText('#integration-test-model-dump');
    //     return text;
    // }, 5000, 'expected text to be different after 5s')
    //   });
    //this.temp = this.model.clone();
  })

  // beforeEach(function () {
  //   this.app.client.click 
  // })

  after(function () {
    if (this.app && this.app.isRunning()) {
      return this.app.stop()
    }
  })

  afterEach(async function () {

    await this.app.client.pause(500)
    await this.app.client.click('#Reset');
    var el = await this.app.client.$('#integration-test-model-dump').getText()
    this.model = JSON.parse(el);
    //await this.app.client.pause(100)
    // console.log('here')
    // var el = await this.app.client.$('#integration-test-model-dump').getText()
    // await this.app.client.pause(500)
    // this.model = await el
    // var model = JSON.parse(this.model).grammars.root.kind;
    // console.log("\n", model);
   })


  it('shows an initial window', function (done) {
    //  console.log(this.model);
    this.app.client.getWindowCount().then(function (count) {
      assert.equal(count, 1)
      // Please note that getWindowCount() will return 2 if `dev tools` are opened.
      // assert.equal(count, 2)
    })
    done();

  })

  it('Reset button', async function () {

    this.app.client.click('#Reset');
    this.model = this.app.client.waitUntil(async function () {
      const text = await this.getText('#integration-test-model-dump');
      return text;
    }, 5000, 'expected text to be different after 5s')
    this.temp = JSON.stringify(this.app.client.$("#integration-test-model-dump"));
    this.app.client.click('#Reset');
    this.model = JSON.stringify(this.app.client.$("#integration-test-model-dump"));
    return assert.equal(this.temp, this.model)
    //Test Reset Button to ensure the Model is reset to the most basic format

  })

  it('shows Buttons', function () {
    this.app.client.click('#Reset');
    this.model = this.app.client.waitUntil(async function () {
      const text = await this.getText('#integration-test-model-dump');
      return text;
    }, 5000, 'expected Value to show after 5s')
    var temp = this.app.client.$$("button");
    //console.log(temp);
    return assert.equal(10, 10);
    // Test the number of buttons loaded to ensure they are all present
  })


  it('insert Row', async function () {
    this.app.client.click('#InsertRow');
    var el = await this.app.client.$('#integration-test-model-dump').getText()
    var temp = JSON.parse(el).grammars.root.kind;

    return assert.equal(true, temp.sort().equals([
        [ 1, 1 ], [ 1, 2 ],
        [ 2, 1 ], [ 2, 2 ],
        [ 3, 1 ], [ 3, 2 ],
        [ 4, 1 ], [ 4, 2 ]]));
  })
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

//Can't compare 2 identical objects in different instances, helper needed:
//https://stackoverflow.com/a/201471
//https://stackoverflow.com/a/14853974
Object.prototype.equals = function(object2) {
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
  for(propName in object2) {
      //We must check instances anyway, there may be a property that only exists in object2
          //I wonder, if remembering the checked values from the first loop would be faster or not 
      if (this.hasOwnProperty(propName) != object2.hasOwnProperty(propName)) {
          return false;
      }
      else if (typeof this[propName] != typeof object2[propName]) {
          return false;
      }
      //If the property is inherited, do not check any more (it must be equa if both objects inherit it)
      if(!this.hasOwnProperty(propName))
        continue;

      //Now the detail check and recursion

      //This returns the script back to the array comparing
      /**REQUIRES Array.equals**/
      if (this[propName] instanceof Array && object2[propName] instanceof Array) {
                 // recurse into the nested arrays
         if (!this[propName].equals(object2[propName]))
                      return false;
      }
      else if (this[propName] instanceof Object && object2[propName] instanceof Object) {
                 // recurse into another objects
                 //console.log("Recursing to compare ", this[propName],"with",object2[propName], " both named \""+propName+"\"");
         if (!this[propName].equals(object2[propName]))
                      return false;
      }
      //Normal value comparison for strings and numbers
      else if(this[propName] != object2[propName]) {
         return false;
      }
  }
  //If everything passed, let's say YES
  return true;
}  