Version 1 beta
This is a modified version of the UCI interface for standard chess, but some things have been removed for ease of implementation purposes. The goal is for common UCI operations to be implemented, so ULCI clients can be used with regular UCI GUIs

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

Server to client:
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
  In debug mode the client should send additional infos to the server, e.g. with the "info string" command, to help debugging, e.g. the commands that the client has received etc.
  This mode should be switched off by default and this command can be sent any time, also when the client is thinking.

* isready
  This is used to synchronize the client with the server. When the server has sent a command or multiple commands that can take some time to complete,
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
  If the server hasn't sent a "ucinewgame" before the first "position" command, the client shouldn't expect any further ucinewgame commands as the server is probably not supporting the ucinewgame command.
  So the client should not rely on this command even though all new servers should support it.
  As the client's reaction to "ucinewgame" can take some time the server should always send "isready" after "ucinewgame" to wait for the client to finish its operation.

* position [fen <fenstring> | startpos ] moves <move1> .... <movei>
  Set up the position described as an L-FEN on the internal board and play the moves on the internal chess board.
  This position should only contain pieces the client indicates support for.
  If the game was played from the standard chess start position the string "startpos" will be sent
  Note: no "new" command is needed. However, if this position is from a different game than the last position sent to the client, the server should have sent a "ucinewgame" inbetween.

* go
  start calculating on the current position set up with the "position" command.
  There are a number of commands that can follow this command, all will be sent in the same string.
  If multiple limits are sent, the client should respect all of them.
  * searchmoves <move1> .... <movei>
    restrict search to this moves only
    Example: After "position startpos" and "go infinite searchmoves e2e4 d2d4"
    the client should only search the two moves e2e4 and d2d4 in the initial position.
    Must be the last flag in the string.
  * wtime <x>
    white has x msec left on the clock
  * btime <x>
    black has x msec left on the clock
  * winc <x>
    white increment per move in mseconds if x > 0
  * binc <x>
    black increment per move in mseconds if x > 0
  * depth <x>
    search a maximum of x plies
  * nodes <x>
    search a maximum of x nodes
  * mate <x>
    search for a mate in x moves
  * movetime <x>
    search a maximum of x mseconds
  * infinite
    search until the "stop" command. Do not exit the search without being told so in this mode! The server should still handle exiting the search, e.g. in case of a human player.

* stop
  Stop calculating as soon as possible, don't forget the "bestmove" token when finishing the search. Human players may take some time to respond to this.

* info
  The server has additional information for the client
  * string <str>
    The server has a generic message for the client
  * clienterror <str>
    The client's communication has an error
  * servererror <str>
    The server has an error message for the client

* eval
  Returns the static evaluation of the current position, optional for debugging purposes

* bench
  Evaluates a series of test positions, optional for debugging purposes

* quit
  quit the program as soon as possible

Client to server:
--------------

* id
  * version <x>
    This must be sent to indicate to the server the version of ULCI supported.
    If the server does not receive this, it should assume the client is a regular UCI client.
    e.g. "id version 1\n"
  * pieces <x>
    Identifies the pieces supported by the client. The server should not give the client positions containing other pieces. If this is not specified, the server should assume the client only supports the pieces from standard chess.
    e.g. "id pieces kmqcaehuriwbznxlop\n"
  * name <x>
    This must be sent after receiving the "uci" command to identify the client,
    e.g. "id name Shredder X.Y\n"
  * username <x>
    Identifies that the client is a human player with a username. Human players will have differences in behaviour compared to engines.
    e.g. "id username Mathmagician\n"
  * author <x>
    This must be sent after receiving the "uci" command to identify the client,
    e.g. "id author Stefan MK\n"

* uciok
  Must be sent after the id and optional options to tell the server that the client has sent all info and is ready in uci mode.

* readyok
  This must be sent when the client has received an "isready" command and has processed all input and is ready to accept new commands now.
  It is usually sent after a command that can take some time to be able to wait for the client, but it can be used anytime, even when the client is searching,and must always be answered with "isready".

* bestmove <move>
  The client has stopped searching and found the move <move> best in this position. This command must always be sent if the client stops searching, also if there is a "stop" command, so for every "go" command a "bestmove" command is needed!
  Directly before that the client should (unless they are a human player) send a final "info" command with the final search information, so that the server has the complete statistics about the last search.

* info
  The client wants to send information to the server. This should be done whenever one of the info has changed.
  The client can send one or multiple info messages with one info command,
  e.g. "info currmove e2e4 currmovenumber 1" or
        "info depth 12 nodes 123456 nps 100000".
  All info belonging to the pv should be sent together
  e.g. "info depth 2 score cp 214 time 1242 nodes 2124 nps 34928 pv e2e4 e7e5 g1f3"
  I suggest to start sending "currmove", "currmovenumber", "currline" and "refutation" only after one second
  to avoid too much traffic.
  Additional info:
  * depth <x>
    search depth in plies
  * seldepth <x>
    selective search depth in plies, if the client sends seldepth there must also be a "depth" present in the same string.
  * time <x>
    the time searched in ms, this should be sent together with the pv.
  * nodes <x>
    x nodes searched, the client should send this info regularly
  * pv <move1> ... <movei>
    the best line found
  * multipv <num>
    For multi pv mode. For the best move/pv add "multipv 1" in the string when you send the pv.
    In k-best mode always send all k variants in k strings together.
  * score
    * cp <x>
      The score from the client's point of view in centipawns.
    * mate <y>
      mate in y moves, not plies.
      If the client is getting mated use negative values for y.
    * lowerbound
      the score is just a lower bound.
      e.g. "info score lowerbound cp 107"
    * upperbound
      the score is just an upper bound.
  * wdl <w> <d> <l>
    The predicted change of a win, draw or loss from the client's perspective permill.
  * currmove <move>
    currently searching this move
  * currmovenumber <x>
    currently searching move number x, for the first move x should be 1 not 0.
  * hashfull <x>
    the hash is x permill full, the client should send this info regularly
  * nps <x>
    x nodes per second searched, the client should send this info regularly
  * tbhits <x>
    x positions where found in the endgame table bases
  * cpuload <x>
    the cpu usage of the client is x permill.
  * string <str>
    Any string str which will be displayed by the client, if there is a string command the rest of the line will be interpreted as <str>, unless an error argument is supplied.
    Errors:
    * clienterror <str>
      The client has an error message to send to the server, the client may terminate if the error is fatal.
    * servererror <str>
      The client has detected a problem with the server's commands and may terminate if the error is fatal.
  * currline <cpunr> <move1> ... <movei>
    this is the current line the client is calculating. <cpunr> is the number of the cpu if the client is running on more than one cpu. <cpunr> = 1,2,3....
    if the client is just using one cpu, <cpunr> can be omitted.
    If <cpunr> is greater than 1, always send all k lines in k strings together.
    The client should only send this if the option "UCI_ShowCurrLine" is set to true.

* option
  This command tells the server which parameters can be changed in the client.
  This should be sent once at client startup after the "uci" and the "id" commands if any parameter can be changed in the client.
  The server should parse this and build a dialog for the user to change the settings.
  Note that not every option needs to appear in this dialog as some options like "Hash" are better handled elsewhere or are set automatically.
  If the user wants to change some settings, the server will send a "setoption" command to the client.
  Note that the server need not send the setoption command when starting the client for every option if it doesn't want to change the default value.
  For all allowed combinations see the examples below, as some combinations of this tokens don't make sense.
  One string will be sent for each parameter.
  * name <id>
    The option has the name id.
    Certain options have a fixed value for <id>, which means that the semantics of this option is fixed.
    Usually those options should not be displayed in the normal client options window of the server but get a special treatment. All those certain options have the prefix "UCI_" except for the
    first 6 options below. If the server gets an unknown Option with the prefix "UCI_", it should just
    ignore it and not display it in the client's options dialog.
    * <id> = Hash, type is spin
      the value in MB for memory for hash tables can be changed,
      this should be answered with the first "setoptions" command at program boot
      if the client has sent the appropriate "option name Hash" command,
      which should be supported by all clients!
      So the client should use a very small hash first as default.
    * <id> = MultiPV, type spin
      the client supports multi best line or k-best mode. the default value is 1
    * <id> = UCI_ShowCurrLine, type check, should be false by default,
      the client can show the current line it is calculating. see "info currline" above.
    * <id> = UCI_Opponent, type string
      With this command the server can send the name, title, elo and if the client is playing a human
      or computer to the client.
      The format of the string has to be [GM|IM|FM|WGM|WIM|none] [<elo>|none] [computer|human] <name>
      Examples:
      "setoption name UCI_Opponent value GM 2800 human Gary Kasparov"
      "setoption name UCI_Opponent value none none computer Shredder"
    * <id> = UCI_EngineAbout, type string
      With this command, the client tells the server information about itself, for example a license text,
      usually it doesn't make sense that the server changes this text with the setoption command.
      Example:
      "option name UCI_EngineAbout type string default Shredder by Stefan Meyer-Kahlen, see www.shredderchess.com"
    * <id> = UCI_SetPositionValue, type string
      the server can send this to the client to tell the client to use a certain value in centipawns from white's
      point of view if evaluating this specifix position.
      The string can have the formats:
      <value> + <fen> | clear + <fen> | clearall
  * type <t>
    The option has type t. There are 5 different types of options the client can send.
    * check
      a checkbox that can either be true or false
    * spin
      a spin wheel that can be an integer in a certain range
    * combo
      a combo box that can have different predefined strings as a value
    * button
      a button that can be pressed to send a command to the client
    * string
      a text field that has a string as a value, an empty string has the value "<empty>"
  * default <x>
    the default value of this parameter is x
  * min <x>
    the minimum value of this parameter is x
  * max <x>
    the maximum value of this parameter is x
  * var <x>
    a predefined value of this parameter is x
  Examples:
  Here are 5 strings for each of the 5 possible types of options
    "option name Nullmove type check default true\n"
    "option name Selectivity type spin default 2 min 0 max 4\n"
    "option name Style type combo default Normal var Solid var Normal var Risky\n"
    "option name UCI_EngineAbout type string default Shredder by Stefan Meyer-Kahlen, see www.shredderchess.com\n"
    "option name Clear Hash type button\n"
