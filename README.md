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

Each grammar (cell) either contains a plain value itself, or contains a nested table of grammars.

## Other ideas about data-model

each cell is maybe represented as a bi/quad/hexa-directional linked list of "cell" objects which are linked to their neighbors on all right-down, all edges and/or corners. and then another "link".
also a good idea to think of nesting could be a seventh arrow pointing inwards to the 

```
       ___ 
      |__|--      a corner cell, with only south
       | \

      \_|_/ 
    --|___|--     a cell in the middle with all it's  neighbors around it
      / | \

```

we could potentially continue with the current map-based implementation, but then the "grammars" will have 6 nullable "neighbor" (and maybe one child and one parent) !reference(s) to the corresponding grammar objects if they exist.
it's kind of nifty because any permutation of neighbors is possible.

this would be an interesting way to find 

