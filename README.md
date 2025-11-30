# 🎄 Advent Of Time

This is the source code for the **Advent Of Time** webapp.
The **Advent Of Time** is an advent type of game that takes place in December.
Every day there will be a different picture and your goal is to try to guess the exact time it was taken at.
You get points based on how well your perform.

The webapp relies on the [rtfw-http-rs](https://github.com/RTFW-rs/rtfw-http-rs) HTTP server.

A live version of this webapp can be found at [aot.coko7.fr](https://aot.coko7.fr)

## Configuration

Use [config.toml](./config.toml) to configure things like the hostname, OAuth2, etc.

## Run it

Simply `cd` to the root dir of the project and type:
```console
cargo run
```
This should start the server on the specific hostname you configured.
The default is: http://127.0.0.1:7878
