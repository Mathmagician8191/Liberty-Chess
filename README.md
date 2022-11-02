# Liberty-Chess

The sequel to Totally Normal Chess, now written in Rust

This is currently in a pre-alpha state where key functionality (e.g. move validation, loading L-FENs and moving pieces) is only partially implemented.

TODO for version 1.0:
- Serializing a board state to an L-FEN
- Promotion for help screen
- Friendly fire mode
- Support for chess engines

For information on the copyright of the images, see COPYING.md.

For information on the types of pieces and their moves, see Pieces.md

For information on L-FEN, see Standards.md

## Build Instructions:

`cargo build --release -p liberty_chess_gui`

The resulting binary will be placed in `target/release/liberty_chess_gui`
