# Update Game

The client makes a request to receive updates on a specific game. Every time an event happens, such as a client joining the game or toggling its ready state, the client making the request is notified. The server responds with the game details. This call blocks the handling thread until an event occurs.

## Path

```json
    /update_game
```

## Request

```json
{
    "game_id": "A49950AA047C2292E989E368A97A3AAA"
}
```

## Response

```json
{
    "id" : "A49950AA047C2292E989E368A97A3AAA",
    "name": "Patrick's game",
    "host" : "Patrick#3BBE",
    "gamemode" : "Domination",
    "map" : "Kobra",
    "max_players" : 12,
    "player_amount": 6,
    "players" :
    [
        {
            "username" : "Matthew#3AAE"
        },
        {
            "username" : "Patrick#3BBE"
        }
    ]
}
```