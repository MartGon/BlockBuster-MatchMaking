# Game details

Requests detailed information of a given game. Request is made by game id.

## Path

```
    /game_details
```

## Request

```json
{
    "game_id" : "A49950AA047C2292E989E368A97A3BB1",
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