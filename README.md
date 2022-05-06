# Yairc

## Yairc == Yet Another IRc Client

Just a simple toy project I've put together to play a bit more with Rust and experiment
with different things. It's currently only partially functional but does allow you to connect
to an IRC server using the command line.

There's no real goals here. It's mostly for my own benefit to see how hard or easy some things
are.

Some of the things I'm experimenting with currently:

- Multi-threaded programming in
- Building a TUI
- Building an IRC protocol parser
- Experimenting with building a Rust binary and distributing

Some other things I might try to do later:

- Seeing how a plugin architecture might work (integrate Lua maybe?)
- Allowing users to customize the TUI
- Deeper TUI interface (it's pretty shallow right now)
- OS notifications

### Usage

**Yairc** is very simple currently. It only allows you to connect to a single IRC server, and a single room at
a time.

```sh
$ yairc <server_name> <channel_name> <nick>
```