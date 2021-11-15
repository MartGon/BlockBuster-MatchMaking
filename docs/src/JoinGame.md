
# Join Game

The client requests to join a specific game by id. The server responds with the game details

## Path
```
    /join_game
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