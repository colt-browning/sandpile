Implementation of the [sandpile model](https://en.wikipedia.org/wiki/Abelian_sandpile_model) in Rust.

Example calls:

`cargo run --release finite 60x50 ascii+png id out/id.png`

`echo "2 6, 8 36, 12 13, 17 10." | cargo run --release finite 40x40 ascii+png add all-3 read_list out/tropical.png`

The underlying graph is always a rectangular grid. Various boundary conditions are available:

* `finite` grid with sink all around the grid;
* `toroidal` grid with sink at the top-left node.

The following targets are available:

* `id`: neutral (identity) element of the sandpile group;
* `read`: read sandpile from keyboard using the same format as the output;
* `read_list`: read list of chipd from the keyboard as pairs of coordinates: `0 0, 0 1, 0 1, 2 1.`;
* `all-N`: a sandpile with N chips in every node;
* `add`: get two sandpiles according to further targets and then add them together.

Licensed under [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) at your option.
