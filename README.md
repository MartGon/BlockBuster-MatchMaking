# BlockBuster Match Making Server 

## Description

This repository contains the Match Making Server project for the multiplayer voxel first person shooter [BlockBuster](https://github.com/MartGon/BlockBuster).

## How to use

### Requirements

The Block Buster Match Making Server is in charge of launching a Block Buster Server per each game that is played through it. Therefore, in order to use it properly, you'll need to build the server application by cloning [this repo](https://github.com/MartGon/BlockBuster), following the builds steps described there, but replacing step 3 with this:

`cmake .. -D BUILD_APPS=0`

It should be noted, that you don't need OpenGL or SDL development libraries in order to build the server. The *-D BUILD_APPS=0* line is used for this purpose; to avoid building the client and editor, which depend on those libraries.

### Configuration

If the machine you are using to run this server is behind a firewall or a NAT router, note that you should whitelist or port forward UDP connections on ports 8000 to 8400, which are used by the game server after a match starts.

### Launching

In order to launch the Match Making Server, you'll have to provide the following parameters:

- Address (-a, --address): The ip address to listen for request. This is usually 0.0.0.0, in case you'd like the server to listen in all the available interfaces
- Port (-p, --port): Port number to listen on.
- Game Address (-g, --game-address): **IMPORTANT** This is the address that will be provided to clients when connecting to a game server. This should be your routers public ip address if running behind NAT and using port forwarding.
- Server Path (-s, --server-path): Path to server executable file. When a game is started, this file will be run. This should be the file you compiled earlier.
- Maps Folder (-m, --maps-folder): Path to the folder which will hold the maps files that players may upload.

## About

My second project made in Rust, and probably still really far away from ideal Rusty code.
