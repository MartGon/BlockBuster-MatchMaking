# Login

The user logs in by requesting an username. The server responds with 200 Ok and with its generated unique id. This could be a SHA-256 digest of the current data, the username and a random number. The last or first digits of the digest could be combined to create a unique username. E.g. **Matthew#3AAE**

## Path

```
    /login
```

## Request

```json
{
    "username" : "Matthew"
}
```

## Response
```json
{
    "username" : "Matthew#3AAE"
}
```