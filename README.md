# Integrated Spreadshee Environment

Experimental speadsheet-based interface for structured programming based on "structured grammars".

Think of it as a grid-based terminal of sorts. It's pretty much the same as regular spreadsheets but then allows for each cell to represent a table in itself (ad-infinitum).

The idea came when i realized what the terminal represents as something of the base-level substrate for interacting with computers. 
The terminal is something of a relic of history, being an emulator of pre-CRT computers that utilized teletypes to display the output of computations. 

The spreadsheet metaphor is appealing because of it's seeming approachability & readability to the average user and it's unique suitability for [homoiconicity](https://en.wikipedia.org/wiki/Homoiconicity). 
In a way, spreadsheets present themselves as the closest non-textual form of a LISP-like programming language, in that the are syntax-less in themselves but might allow for encoding a variety of other syntaxes 
in them, especially as enabled by the nested table idea. It effectively presents itself as a LISP without parentheses that is edited outside of text files, and has the same liveness properties of spreadsheets.

The structured/grid-based layout is appealing because it enables arbitrary structuring of notation similarly to text, and the nesting enables there to be heirarchy to the individual notations or ["grammars"](https://en.wikibooks.org/wiki/Introduction_to_Programming_Languages/Grammars) 
that make up a specfic language. 

Given that the terminal serves as both a REPL for interacting with the computer programs, as well as a platform for editors for creating/editing programs, i believe this interface can serve a similar purpose. Spreadtable aims to become a common substrate for editing structured programs, much like teletype emulators are something of a common substrate for editing free-form text.

A more extensive design document is available [here](https://docs.google.com/document/d/1Sq8BbzPhFWX8j5_7Rtlu1IfgGYJRWQ3Sb5A__jI3AE4/edit?usp=sharing)

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

