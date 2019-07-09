# Scrap

A simple CTF platform written in Rust.

Scrap is designed to be as fast and lightweight as possible. It compiles into a single Rust binary and can handle many thousands of teams. Furthermore, Scrap's browser webpage is completely free of JavaScript. Due to various design decisions, there are a few constraints:

* Only dynamic scoring is supported.
* There is a maximum of 64 challenges.
* Teams lack email verification.
* Registration lacks captchas.

Registration can be rate limited through a reverse proxy if necessary.

### Getting Started

Download the [latest version](https://github.com/pearl/scrap/releases/latest) from the releases page and run it:

```bash
./scrap --port 8000 --repo ./repository --static ./static --uri postgres://user:pass@host/db
```

## Deployment

### Repository

Scrap is centered around a central repository that specifies challenges and files. An example is available [here](https://github.com/pearl/scrap/tree/master/examples/respository).

```
repository
├── ctf.toml
├── first
│   └── challenge.toml
├── second
│   └── challenge.toml
└── third
│   └── challenge.toml
└── junk
    └── trash
```

Scrap requires a single `ctf.toml`, as well as a `challenge.toml` for each challenge. All other files and directories are ignored.

#### ctf.toml

`ctf.toml` must be in the base directory.

```toml
# HTML title element text
title = "MyCTF"

# Homepage Markdown/HTML
home = """
# Welcome to MyCTF!
Enjoy our many *challenges*."""

# CTF start time
# If removed, infinitely in the past
start = 2000-01-01T00:00:00Z

# CTF stop time
# If removed, infinitely in the future
stop = 2100-01-01T00:00:00Z
```

Challenges, scoreboard, and flag submission remain unavailable until the time specified by `start`. Flag submission becomes unavailable once the time specified by `stop` is reached.

#### challenge.toml

Each `challenge.toml` must be exactly two levels below the base directory. The name of the intermediate challenge directory is irrelevant.

```toml
# Unique identifier
slug = "caesar_cipher"

# Title text
title = "Caesar Cipher"

# Author text
author = "username"

# Description Markdown/HTML
description = """
I encrypted a [message](ciphertext.txt) \
with a [Caesar cipher](encrypt.py)!"""

# Tag text
tags = [ "crypto", "classical" ]

# Paths to files anywhere in the challenge directory
files = [ "path/to/ciphertext.txt", "encrypt.py" ]

# Challenge flag
flag = "flag{}"

# Challenge status
enabled = true
```

Scrap will either update or add a challenge depending on whether `slug` exists in the database.

Paths in `files` can traverse directories, but must have unique filenames. These files can be referred to by filename in `description` for links.

Challenges with `enabled` set to `true` are displayed, open to flag submission, and used in calculating score. Challenges with `enabled` set to `false` are not, but maintain state for future toggling.

### Database

Scrap requires a PostgreSQL server with the `pgcrypto` extension enabled:

```sql
create extension pgcrypto;
```

### Static Files

Scrap uses a static directory, which must be served separately at `/static`. This can be used for serving stylesheets and favicons. Scrap will also create a subdirectory named `files` to hold challenge files.

```
static
├── style.css
├── favicon.ico
```

### Running

Scrap requires four arguments at runtime:

- `port` Server port
- `repo` Path to repository
- `static` Path to static directory
- `uri` PostgreSQL database URI

```bash
./scrap --port 8000 --repo ./repository --static ./static --uri postgres://user:pass@host/db
```

### Signals

Scrap supports graceful reloading on `SIGUSR1`. Send the signal to reload the CTF and challenge configuration from the repository.

## Customization

### Compiling

At the moment, Scrap requires nightly Rust.

```bash
cargo +nightly build --release
```

### Styling

Scrap assumes that you have included `style.css` and `favicon.png` within the static directory. An example [SCSS](https://sass-lang.com/documentation/syntax) file is available [here](https://github.com/pearl/scrap/blob/master/examples/style.scss) to function as documentation for the classes.

### Scoring

The dynamic scoring formula can be changed by modifying the `value` function in `scrap.sql`.
