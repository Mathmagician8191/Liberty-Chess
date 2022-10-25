# Liberty-Chess

The sequel to Totally Normal Chess, now written in Rust

This is currently in a pre-alpha state where key functionality (e.g. move validation) is not yet implemented and many other features (e.g. loading L-FENs and moving pieces) is only partially implemented.

Implemented so far:
- Loading and rendering pieces from an L-FEN (remaining L-FEN functionality missing)
- Basic piece moving functionality (accounts for none of the special cases)

For information on the copyright of the images, see COPYING.md.

For information on the types of pieces and their moves, see Pieces.md

For information on L-FEN, see Standards.md

## Build Instructions:

`cargo build --release -p liberty_chess_gui`

The resulting binary will be placed in `target/release/liberty_chess_gui`
