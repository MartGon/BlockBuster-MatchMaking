# Create Game

The clients sends a request to create a game. This requests includes the game's name, map, number of players, etc. The server simply responds with 200 Ok or 400 Bad Requests and with a game id.

## Path
```http
    /create_game
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
{
    "id" : "A49950AA047C2292E989E368A97A3BB1",
}
```