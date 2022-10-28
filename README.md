# Liberty-Chess

The sequel to Totally Normal Chess, now written in Rust

This is currently in a pre-alpha state where key functionality (e.g. move validation, loading L-FENs and moving pieces) is only partially implemented.

TODO for version 1.0:
- Reading the other fields of an L-FEN
- Serializing a board state to an L-FEN
- Castling
- Pawn moves and en passant
- Testing for exposing the king to attack/check
- Credits screen
- Castling, check and en passant for help screen
- Support for chess engines

For information on the copyright of the images, see COPYING.md.

For information on the types of pieces and their moves, see Pieces.md

For information on L-FEN, see Standards.md

## Build Instructions:

`cargo build --release -p liberty_chess_gui`

The resulting binary will be placed in `target/release/liberty_chess_gui`
