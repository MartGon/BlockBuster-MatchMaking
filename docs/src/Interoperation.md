# Interoperation

There's need for some coordination with the match-making server and the game server. There are two possibilities:

- Game Server hosted on Match-Making server
- Game Server hosted on Client

## Game Server hosted on Match-Making server

This is the first option to be implemented. When a player sends a CreateGame request, the server launches a game server which will listen in a specific port. That port is given to whatever client joins that game (with a JoinGame request), along with an IP Address, the match-making server's IP in this case.

Clients must open a connection against the game server. It should be open while they're waiting on the lobby.

When the host sends a StartGame request, the match-making server orders the game server (using a socket or IPC) to start the game. From that moment onwards, all communication is done between clients and the game server.

## Game Server hosted on Client

To be defined.

Will need NAT punch hole