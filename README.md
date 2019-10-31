# Integrated Spreadshee Environment

Experimental speadsheet-based interface for structured programming based on "structured grammars".

# Developing

install dependencies using:
```
cargo install cargo-web
```

build and release:
```
cargo web build
```

run:
```
cargo web start

cargo web start --release
```

# Documentation

The frontend of this project aims to use Elm's functional reactive architecture to build an adaptive, nestable grid layout. 

Data model consists of a Map of coordinates (as strings) to structs representing the individual "Grammars" that make up a 
representation of a program.

Each grammar (cell) is either a static text value, an input box, or a nested table of grammars.