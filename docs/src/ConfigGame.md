# Config Game

The clients sends a request to change the config of an existing game. This requests includes the game's name, map, etc. The server simply responds with 200 Ok or 400 Bad Request.

## Path
```http
    /config_game
```

## Request

```json
{
    "name": "Matthew's game",
    "gamemode" : "Team DeathMatch",
    "map" : "Kobra",
    "max_players" : 8,
    "players": 4
}
```

## Response

```json
(empty)
```