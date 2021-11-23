# ToggleReady

Clients can use this to toggle their ready state. It only works when the player is on a game. Returns 200 Ok or 400 Bad with an error string.

## Path

```
    /ready
```

## Request

```json
    (empty)
```

## Response

```json
{
    "ready" : true
}
```