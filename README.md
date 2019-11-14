# Integrated Spreadsheet Environment

Experimental speadsheet-based interface for structured programming based on "structured grammars".

# Developing

## 
build and release:
```
cargo build
```

run:
```
cargo run
```

`api_proxy` runs at localhost:7878
`frontend_builder` runs at localhost:8000

# Documentation

The frontend of this project aims to use Elm's functional reactive architecture to build an adaptive, nestable grid layout. 

Data model consists of a Map of coordinates (as strings) to structs representing the individual "Grammars" that make up a 
representation of a program.

Each grammar (cell) is either a static text value, an input box, or a nested table of grammars.
