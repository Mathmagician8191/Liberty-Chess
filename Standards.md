# L-FEN

A standard text format for describing positions in Liberty Chess.

An L-FEN is broken up into several fields that are space separated.
Some fields are optional, but if a field is present all previous ones must also be present.
Optional fields also present in the standard FEN (those up to Fullmove number) must always be present in output to maintain backwards compatibility unless the user indicates otherwise.

This format is not final and subject to change.

## Piece definitions

The ranks are separated by the slash character "/".

Within each rank, pieces are specified by letters (uppercase for White, lowercase for Black), and empty squares are specified by a number indicating the number of empty squares present.

The number of files in each rank must be consistent.

## Side to move

"w" indicates white to move, "b" indicates black to move.

## Castling rights

Represents both sides' remaining abilities to castle.

If no castling rights exist, represent as "-".
If castling rights do exist, output 1 letter per castling right as follows:
- K: White can castle kingside
- Q: White can castle queenside
- k: Black can castle kingside
- q: Black can castle queenside

The letters should be in order shown above.

This field is optional, the default value is "-".

## En passant target square

Represents squares valid for en passant capture.

If no square is valid for en passant capture, the output is "-".

If a single square is valid for en passant capture, the output is that square represented using the coordinate representation.

If several squares in the same file are valid for en passant capture, the output is the square with the lowest rank, using the coordinate representation, a dash ("-"), and the highest rank of the squares.

This field is optional, the default value is as "-".

## Halfmove clock

The number of halfmoves since the last capture or pawn move.

This field is optional, the default value is 0.

## Fullmove number

The number of full moves the game has lasted.
Starts at 1 and increments after every Black move.

This field is optional, the default value is 1.

## Misc configuration

This field contains the following comma separated fields:

The number of squares a pawn can move on its first move, default 2.

The maximum row a pawn can move multiple squares from, default 2.

The row castling occurs on, default 1.

The column of the queenside castling piece, default 1.

The column of the kingside castling piece, defaults to the board width.

If some fields are missing, they use the default value.

# Coordinate representation

This format is made of 2 parts, with no separation.

The default orientation of the board is with White pawns moving upwards.

## File

The file is specified using a letter or letters.

a-z covers the first 26 ranks, where a is the file furthest to the left of the board. From there, it goes aa-az, ba-bz etc, adding more letters if necessary.

## Rank

The rank is specified as a number, with 1 being the promotion rank for Black pawns and the rank increasing up the board.
