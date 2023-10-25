Still being worked on, subject to change
This is a modified version of the regular chess UCI interface, but some things have been removed for ease of implementation purposes. The goal is for common UCI operations to be implemented, so ULCI clients can be used with regular GUIs

ULCI (Universal Liberty Chess Interface) is the standard method for server-client communication, such as with a server communicating with an client, or a multiplayer server and client.

Description of the ULCI interface
Based on the UCI interface reference found at https://github.com/rooklift/nibbler/blob/master/files/misc/uci.txt

* The specification is independent of the operating system. For Windows, the client is a normal exe file, either a console or "real" windows application.

* all communication is done via standard input and output with text commands,

* The client should boot and wait for input from the server, the client should wait for the "isready" or "setoption" command to set up its internal parameters as the boot process should be as quick as possible.

* the client must always be able to process input from stdin, even while thinking.

* all command strings the client receives will end with '\n',
  also all commands the server receives should end with '\n',
  Note: '\n' can be 0x0d or 0x0a0d or any combination depending on your OS.
  If you use client and server in the same OS this should be no problem if you communicate in text mode, but be aware of this when for example running a Linux client in a Windows server.

* arbitrary white space between tokens is allowed
  Example: "debug on\n" and "   debug     on  \n" and "\t  debug \t  \t\ton\t  \n" all set the debug mode of the client on.

* The client will always be in forced mode which means it should never start calculating or pondering without receiving a "go" command first.

* Before the client is asked to search on a position, there will always be a position command to tell the client about the current position.

* If the client receives a command that it does not recognise, it should ignore it

* If the client receives a command which is not supposed to come, for example "stop" when the client is not calculating, it should also just ignore it.

`startpos` refers to the L-FEN position `rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1`

Move format:
------------

The move format is in long algebraic notation.
A nullmove from the client to the server should be sent as 0000.
Examples: e2e4, e7e5, e1g1 (white short castling), e7e8q (for promotion), g10g7 (a move with more than 4 characters)

server to client:
--------------

These are all the command the client gets from the interface.

* uci
  Tell client to use the uci (universal chess interface), this will be sent once as a first command after program boot to tell the client to switch to uci mode.
  The server does not need to identify itself as supporting ULCI, the client should be capable of dealing with both normal UCI and ULCI without special setup.
  After receiving the uci command the client must identify itself with the "id" command and send the "option" commands to tell the server which client settings the client supports if any.
  After that the client should send "uciok" to acknowledge the uci mode.
  If no uciok is sent within a certain time period, the client task will be killed by the server.

* debug [ on | off ]
  Switch the debug mode of the client on and off.
  In debug mode the client should send additional infos to the GUI, e.g. with the "info string" command, to help debugging, e.g. the commands that the client has received etc.
  This mode should be switched off by default and this command can be sent any time, also when the client is thinking.

* isready
  This is used to synchronize the client with the GUI. When the GUI has sent a command or multiple commands that can take some time to complete,
  This command can be used to wait for the client to be ready again or to ping the client to find out if it is still alive.
  E.g. this should be sent after setting the path to the tablebases as this can take some time.
  This command is also required once before the client is asked to do any search to wait for the client to finish initializing.
  This command must always be answered with "readyok" and can be sent also when the client is calculating in which case the client should also immediately answer with "readyok" without stopping the search.

* setoption name <id> [value <x>]
  This is sent to the client when the user wants to change the internal parameters of the client. For the "button" type no value is needed.
  One string will be sent for each parameter and this will only be sent when the client is waiting.
  The name and value of the option in <id> should not be case sensitive and can inlude spaces.
  The substrings "value" and "name" should be avoided in <id> and <x> to allow unambiguous parsing, for example do not use <name> = "draw value".
  Here are some strings for the example below:
    "setoption name Nullmove value true\n"
    "setoption name Selectivity value 3\n"
    "setoption name Style value Risky\n"
    "setoption name Clear Hash\n"

* ucinewgame
  This is sent to the client when the next search (started with "position" and "go") will be from a different game. This can be a new game the client should play or a new game it should analyse but also the next position from a testsuite with positions only.
  If the GUI hasn't sent a "ucinewgame" before the first "position" command, the client shouldn't expect any further ucinewgame commands as the GUI is probably not supporting the ucinewgame command.
  So the client should not rely on this command even though all new GUIs should support it.
  As the client's reaction to "ucinewgame" can take some time the GUI should always send "isready" after "ucinewgame" to wait for the client to finish its operation.

* position [fen <fenstring> | startpos ] moves <move1> .... <movei>
  Set up the position described as an L-FEN on the internal board and play the moves on the internal chess board.
  If the game was played from the start position the string "startpos" will be sent
  Note: no "new" command is needed. However, if this position is from a different game than the last position sent to the client, the GUI should have sent a "ucinewgame" inbetween.

* go
  start calculating on the current position set up with the "position" command.
  There are a number of commands that can follow this command, all will be sent in the same string.
  If one command is not sent its value should be interpreted as it would not influence the search.
  * searchmoves <move1> .... <movei>
    restrict search to this moves only
    Example: After "position startpos" and "go infinite searchmoves e2e4 d2d4"
    the client should only search the two moves e2e4 and d2d4 in the initial position.
  * wtime <x>
    white has x msec left on the clock
  * btime <x>
    black has x msec left on the clock
  * winc <x>
    white increment per move in mseconds if x > 0
  * binc <x>
    black increment per move in mseconds if x > 0
  * depth <x>
    search x plies only.
  * nodes <x>
    search x nodes only,
  * mate <x>
    search for a mate in x moves
  * movetime <x>
    search exactly x mseconds
  * infinite
    search until the "stop" command. Do not exit the search without being told so in this mode! The server should still handle exiting the search, e.g. in case of a human player.

* stop
  Stop calculating as soon as possible, don't forget the "bestmove" token when finishing the search

* quit
  quit the program as soon as possible
