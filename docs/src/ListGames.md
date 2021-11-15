# List games

The client requests the list of available games. The request contains different filters. These filters include: maps, number of players, whether the game is full or empty, gamemode, etc.

The server responds with an array of games that match the filters.

## Path

```
    /list_games
```

## Request

```json
{
    "maps" : ["Kobra", "Last Resort"],
    "empty" : false,
    "full" : false
}
```

## Response

```json
{
    "games" : [
        {
            "id" : "A49950AA047C2292E989E368A97A3AAE",
            "name": "Matthew's game",
            "gamemode" : "Team DeathMatch",
            "map" : "Kobra",
            "host" : "Matthew#3AAE",
            "max_players" : 8,
            "players": 4            
        },
        {
            "id" : "A49950AA047C2292E989E368A97A3AAA",
            "name": "Patrick's game",
            "gamemode" : "Domination",
            "map" : "Kobra",
            "hosts" : "Patrick#3BBE",
            "max_players" : 12,
            "players": 6
        }
    ]
}
```
