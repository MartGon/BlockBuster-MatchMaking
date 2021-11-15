# Architecture

## Players

Players are held in a HashMap<K, V>. They hold an id with the game id they're currently in.

```json

{
    "username" : "Matthew#3AAE",
    "current_game_id" : null,
    "ready" : false,
    "level" : 1
}

```

## Games

They are held in a HashMap<K, V>. Contains relevant game data, such as map, mode, host_id, etc.

```json
{
   "id" : "A49950AA047C2292E989E368A97A3AAA",
    "name": "Patrick's game",
    "host" : "Patrick#3BBE",
    "gamemode" : "Domination",
    "map" : "Kobra",
    "max_players" : 12,
}
```