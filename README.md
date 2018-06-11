Implementation of the [sandpile model](https://en.wikipedia.org/wiki/Abelian_sandpile_model) in Rust.

Example call:

`cargo run --release finite 60x50 id ascii+png out/id.png`

The underlying graph is always a rectangular grid. Various boundary conditions are available:

* **finite** grid with sink all around the grid;
* **toroidal** grid with sink at the top-left node.

Currently, the program can only calculate the identity (neutral) element, but the library already can do more.

Licensed under [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) at your option.
