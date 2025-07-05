## Attack/defence values

For a capture to be valid, the attacking piece's attack value must be greater than the defending piece's defence value.

Hierarchy of values: None < Basic < Powerful

## Pawn

<img src="resources/images/WPawn.svg" width="80"/><img src="resources/images/BPawn.svg" width="80"/>

Configuration parameter: max number of squares moved on first move (default 2).

If moving n spaces where n > 2, can be en passant captured as if it had moved anywhere from 1 to n-1 squares.

Otherwise, moves the same as standard chess.

Attack: Powerful \
Defence: None

## Knight

<img src="resources/images/WKnight.svg" width="80"/><img src="resources/images/BKnight.svg" width="80"/>

Moves the same as standard chess.

Attack: Basic \
Defence: None

## Bishop

<img src="resources/images/WBishop.svg" width="80"/><img src="resources/images/BBishop.svg" width="80"/>

Moves the same as standard chess.

Special additional move: El Vaticano
Two bishops of the same colour, two squares apart horizontally or vertically can capture a piece lying in between them.

Attack: Basic \
Defence: None

## Rook

<img src="resources/images/WRook.svg" width="80"/><img src="resources/images/BRook.svg" width="80"/>

Moves the same as standard chess.

Attack: Basic \
Defence: None

## Queen

<img src="resources/images/WQueen.svg" width="80"/><img src="resources/images/BQueen.svg" width="80"/>

Moves the same as standard chess.

Attack: Basic \
Defence: None

## King

<img src="resources/images/WKing.svg" width="80"/><img src="resources/images/BKing.svg" width="80"/>

Moves the same as standard chess.

Attack: Powerful

Can be checked by any piece with at least a Basic attack.

## Archbishop

<img src="resources/images/WArchbishop.svg" width="80"/><img src="resources/images/BArchbishop.svg" width="80"/>

Moves as the combination of a Bishop and a Knight.

Attack: Basic \
Defence: None

## Chancellor

<img src="resources/images/WChancellor.svg" width="80"/><img src="resources/images/BChancellor.svg" width="80"/>

Moves as the combination of a Rook and a Knight.

Attack: Basic \
Defence: None

## Camel

<img src="resources/images/WCamel.svg" width="80"/><img src="resources/images/BCamel.svg" width="80"/>

Jumps 3 spaces in 1 direction and 1 space perpendicular to that.

Attack: Basic \
Defence: None

## Zebra

<img src="resources/images/WZebra.svg" width="80"/><img src="resources/images/BZebra.svg" width="80"/>

Jumps 3 spaces in 1 direction and 2 spaces perpendicular to that.

Attack: Basic \
Defence: None

## Mann

<img src="resources/images/WMann.svg" width="80"/><img src="resources/images/BMann.svg" width="80"/>

Moves like a Queen, but only 1 space at a time.

Attack: Basic \
Defence: None

## Nightrider

<img src="resources/images/WNightrider.svg" width="80"/><img src="resources/images/BNightrider.svg" width="80"/>

Moves like a Knight, but can move multiple times in the same direction in a single move.

Attack: Basic \
Defence: None

## Champion

<img src="resources/images/WChampion.svg" width="80"/><img src="resources/images/BChampion.svg" width="80"/>

Moves like a Queen up to 2 spaces at a time, can jump.

Attack: Basic \
Defence: None

## Centaur

<img src="resources/images/WCentaur.svg" width="80"/><img src="resources/images/BCentaur.svg" width="80"/>

Moves like a Mann or Knight.

Attack: Basic \
Defence: None

## Amazon

<img src="resources/images/WAmazon.svg" width="80"/><img src="resources/images/BAmazon.svg" width="80"/>

Moves like a Queen or Knight.

Attack: Basic \
Defence: None

## Elephant

<img src="resources/images/WElephant.svg" width="80"/><img src="resources/images/BElephant.svg" width="80"/>

Moves like a Mann.

Attack: Powerful \
Defence: Basic

## Obstacle

<img src="resources/images/WObstacle.svg" width="80"/><img src="resources/images/BObstacle.svg" width="80"/>

Can teleport to any square on the board.

Attack: None \
Defence: None

## Wall

<img src="resources/images/WWall.svg" width="80"/><img src="resources/images/BWall.svg" width="80"/>

Moves like an Obstacle.

Attack: None \
Defence: Basic
