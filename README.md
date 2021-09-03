# overcast-to-sqlite

Download your [Overcast](http://overcast.fm) listening history to sqlite. Supports saving podcast feeds and individual episodes.

```
USAGE:
    overcast-to-sqlite --username <USERNAME> --password <PASSWORD> <DB_PATH>

ARGS:
    <DB_PATH>    The sqlite database path to store to

FLAGS:
    -h, --help       Print help information
    -V, --version    Print version information

OPTIONS:
    -p, --password <PASSWORD>    Overcast password
    -u, --username <USERNAME>    Overcast username
```

## Example Usage

```sh
$ overcast-to-sqlite podcasts.db --username=myusername@email.com --password=mypassword
```

## Attribution

This package is heavily inspired the `X-to-sqlite` utilities created by [Simon
Willison](https://simonwillison.net/2019/Oct/7/dogsheep/).

This package was designed to fit nicely in the [dogsheep](https://dogsheep.github.io/) / [datasette](https://github.com/simonw/datasette) ecosystems.