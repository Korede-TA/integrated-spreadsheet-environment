# Integrated Spreadsheet Environment

Experimental speadsheet-based interface for structured programming based on "structured grammars".

# Developing

starting electron app
```
npm run start
```


# Setting up development environment on Windows

1. Install Git Bash (default settings) (https://gitforwindows.org/)
2. Install Rust (https://www.rust-lang.org/tools/install)
3. Install Node.js (12.14.1 LTS) (https://nodejs.org/en/) 
4. Install wasm-pack (https://rustwasm.github.io/wasm-pack/installer/)
5. Install MS Visual Studio (2019, Community) (https://visualstudio.microsoft.com/downloads/)
6. In git bash:
```
git clone https://github.com/Korede-TA/integrated-spreadsheet-environment.git
cd integrated-spreadsheet-environment/
cargo update
npm install
```

# Documentation

The frontend of this project aims to use Elm's functional reactive architecture to build an adaptive, nestable grid layout. 

Data model consists of a Map of coordinates (as strings) to structs representing the individual "Grammars" that make up a 
representation of a program.

Each grammar (cell) is either a static text value, an input box, or a nested table of grammars.
