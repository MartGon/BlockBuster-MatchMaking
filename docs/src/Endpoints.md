# Endpoints

## Login

The user logs in by requesting an username. The server responds with 200 Ok and with its generated unique id. This could be a SHA-256 digest of the current data, the username and a random number. The last or first digits of the digest could be combined to create a unique username. E.g. **Matthew#3AAE**

### Path

```
    /login
```

### Request

```json
{
    "username" : "Matthew"
}
```

### Response
```json
{
    "username" : "Matthew#3AAE"
}
```

## List games

The client requests the list of available games. The request contains different filters. These filters include: maps, number of players, whether the game is full or empty, gamemode, etc.

The server responds with an array of games that match the filters.

### Path

```
    /list_games
```

### Request

```json
{
    "maps" : ["Kobra", "Last Resort"],
    "empty" : false,
    "full" : false
}
```

### Response

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

## Create Game

The clients sends a request to create a game. This requests includes the game's name, map, number of players, etc. The server simply responds with 200 Ok or 400 Bad Requests and with a game id.

### Path
```http
    /create_game
```

### Request

```json
{
    "name": "Matthew's game",
    "gamemode" : "Team DeathMatch",
    "map" : "Kobra",
    "max_players" : 8,
    "players": 4
}
```

### Response

```json
{
    "id" : "A49950AA047C2292E989E368A97A3BB1",
}
```

## Join Game

The client requests to join a specific game by id. The server responds with the game details

### Path
```
    /join_game
```

### Request

```json
{
    "game_id": "A49950AA047C2292E989E368A97A3AAA"
}
```

### Response

```json
{
    "id" : "A49950AA047C2292E989E368A97A3AAA",
    "name": "Patrick's game",
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

## Game details

Requests detailed information of a given game. Request is made by game id.

### Path

```
    /game_details
```

### Request

```json
{
    "game_id" : "A49950AA047C2292E989E368A97A3BB1",
}
```

### Response

```json
{
    "id" : "A49950AA047C2292E989E368A97A3AAA",
    "name": "Patrick's game",
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

