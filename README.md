# overcast-to-sqlite

![crates.io](https://img.shields.io/crates/v/wayback-archiver.svg)

Download your [Overcast](http://overcast.fm) listening history to sqlite. Supports saving podcast feeds and individual episodes.

## Installation

    $ cargo install overcast-to-sqlite

## Usage

```
USAGE:
    overcast-to-sqlite [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Print help information
    -V, --version    Print version information

OPTIONS:
    -a, --auth-file <AUTH_FILE>    Storage location for Overcast credentials [default: auth.json]
    -p, --password <PASSWORD>      Overcast password
    -u, --username <USERNAME>      Overcast username

SUBCOMMANDS:
    archive    Save Overcast feeds/episodes to sqlite
    auth       Authenticate with Overcast
    help       Print this message or the help of the given subcommand(s)
```

## Examples

```sh
$ overcast-to-sqlite auth
$ overcast-to-sqlite archive podcasts.db
```

## Attribution

This package is heavily inspired the `X-to-sqlite` utilities created by [Simon
Willison](https://simonwillison.net/2019/Oct/7/dogsheep/).

This package was designed to fit nicely in the [dogsheep](https://dogsheep.github.io/) / [datasette](https://github.com/simonw/datasette) ecosystems.
