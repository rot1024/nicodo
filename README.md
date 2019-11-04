# nicodo

Niconico-doga Comment Downloader

```
nicodo 0.1.0
rot1024 <aayhrot@gmail.com>
Niconico-doga Comment Downloader

USAGE:
    nicodo [FLAGS] [OPTIONS] <id>

FLAGS:
    -h, --help       Prints help information
    -l, --latest     include latest comments
        --nosave     does not save config file
    -q, --quiet      hide progress
    -V, --version    Prints version information

OPTIONS:
    -d, --date <date>             date: 2019-01-01, 2019-01-01 12:00:00
    -f, --format <format>          [default: xml]  [possible values: json, xml]
    -i, --interval <interval>     interval: 1h, 1d
    -o, --output <output>         output directory path [default: .]
    -e, --end <period end>        period end: 2019-01-01, 2019-01-01 12:00:00, latest, posted+1d, posted+1w
    -s, --start <period start>    period start: 2019-01-01, 2019-01-01 12:00:00, posted, posted+1d, posted+1w
    -u, --session <session>       user_session value in cookie

ARGS:
    <id>    Video id: www.nicovideo.jp/watch/XXXXXXX
```

## Install

Donwload a binary from the GitHub releases page, or manually compile from source code.

## Getting Started

When using nicodo for the first time, the session ID must be set. Once set, it is saved in the configuration file and you can skip this task from the next time.

At first, sign in to nicovideo.jp manually with your browser. Then, you have to find your `user_session` id from the cookie of nicovideo.jp in your browser:

```
user_session=XXXXXXX;
```

Specify the session id as below when using nicodo for the first time:

```sh
nicodo -s XXXXXXX <video id>
```

The configuration file is saved at `~/.local/share/nicodo/nicodo_config.json` (Linux), `~/Library/Application Support/nicodo/nicodo_config.json` (MacOS) or `C:\Users\hoge\AppData\Local\nicodo\nicodo_config.json`. (Windows)

If you cannot fetch comments because of invalid auth, remove the configuration file and renew the session ID.

## Usage

Specify `<video id>` by extracting the XXXXXXX part of the video URL as shown below:

```
https://www.nicovideo.jp/watch/XXXXXXX
```

### Fetch latest comments

```sh
# save latest comments as XML format
nicodo <video id>
# save latest comments as JSON format
nicodo -f json <video id>
```

### Fetch past comments

```sh
# 2010-01-01 00:00:00
nicodo -d 2010-01-01 <video id>
# 2010-01-01 12:00:00
nicodo -d 2010-01-01 12:00:00 <video id>
# post date
nicodo -d posted <video id>
# 1 hour after post date
nicodo -d posted+1h <video id>
# 1 day after post date
nicodo -d posted+1d <video id>
# 1 week after post date
nicodo -d posted+1w <video id>
```

### Fetch all comments within the period

```sh
# fetch comments in 6-hour increments for 1 week from the post date
nicodo -s posted -e posted+1w -i 6h <video id>
# include latest comments
nicodo -s posted -e posted+1w -i 6h -l <video id>
```
