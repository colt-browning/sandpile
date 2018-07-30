Implementation of the [sandpile model](https://en.wikipedia.org/wiki/Abelian_sandpile_model) in Rust.

Example calls:

* Identity element:
`cargo run --release finite 60x50 ascii+png id out/id.png`
* Verify that `inverse` indeed gives inverse:
`cargo run --release finite 10 eq id add inverse dup all-3`
* [Tropical curves](https://en.wikipedia.org/wiki/Tropical_geometry):
`echo "2 6, 8 36, 12 13, 17 10." | cargo run --release finite 40x40 ascii+png add all-3 read_list out/tropical.png`
* [OEIS A256046](https://oeis.org/A256046) (see also [A256045](https://oeis.org/A256045)):
`for ($n = 2; $n -lt 9; $n++) { cargo run --release finite $n order all-2 }` (PowerShell)
or
`for n in {2..8}; do cargo run --release finite ${n} order all-2; done` (bash)
* [OEIS A249872](https://oeis.org/A249872) (see also [A293452](https://oeis.org/A293452)):
`for ($n = 1; $n -lt 10; $n++) { cargo run --release torus $n topplings all-4 }`
or
`for n in {1..9}; do cargo run --release torus ${n} topplings all-4; done`

The underlying graph is always a rectangular grid. Various boundary conditions are available:

* `finite` grid with sink all around the grid;
* `toroidal` grid with sink at the top-left node.

The following output options are available (all but the last two can be combined with each other):

* `ascii`: plaintext to standard output;
* `png`: image to a file specified by the final command line argument;
* `topplings`: how many topplings did the sandpile take to stabilize during the final operation;
* `order`: the order of the recurrent sandpile;
* `recurrent`: check whether the sandpile is recurrent;
* `eq`: check whether two sandpiles are equal.

The following targets are available:

* `id`: neutral (identity) element of the sandpile group;
* `read`: read sandpile from standard input using the same format as the output;
* `read_list`: read list of chipd from the keyboard as pairs of coordinates: `0 0, 0 1, 0 1, 2 1.`;
* `all-N`: a sandpile with N chips in every node;
* `add`: get two sandpiles from the stack according to further targets and then add them together;
* `inverse`: get a sandpile from the stack and take its inverse if it is recurrent;
* `dup`: get a sandpile and put it back twice.

Licensed under [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) at your option.
