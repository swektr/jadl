# Jadl
**J**apanese **A**udio **D**own**l**oad

Command line tool to download mp3 audio readings of Japanese words.  Works on Linux, maybe MacOS. Very likely won't work on Windows.

This is my first Rust program! It's a rewrite of an old Bash script I made that does the same thing.  A lot of "square peg in a round hole" going on with this code, but it works.  I'm using this more as a way to learn about Rust.


# Dependencies 
**Rust**: built with version "1.70.0", but older versions likely work fine.

**libmpv**: version "2.X" (mpv version >= "0.35.0")  

**curl**: probably any version released in the past 15 years. (Note: this program doesn't utilize 'libcurl' but just runs the command itself as a sub process.)

# Building
Run: `cargo build`

# Usage
Run program like this `jadl [OPTIONS] <KANJI> <KANA>`

<u>Arguments:</u>
* `KANJI` (required) The word written in kanji.
* `KANA` (required) The word's reading written in kana.

<u>Options:</u>
* `-f, --force` -- Overwrite existing files.
* `-a, --anki` -- Download file to anki's `collection.media` directory.
* `-h, --help` -- Print usage help.

# Configuration
 The download directories can be configured using a config file located at: `$XDG_CONFIG_HOME/jadl/config.toml`. If `XDG_CONFIG_HOME` is not set, then `$HOME/.config/jadl/config.toml` is used.

 If no config file is present then files are downloaded to your home directory.

Here's an example of a config.toml file
```toml
dest_dir = "/path/to/download/desired/location"
anki_dir = "/path/to/your/collection.media/" #Optional 
```

:warning: **WARNING:** Must use absolute paths, environment variables will not be parsed and tilde `~` home will not work either.

# Potential issues
Since the libmpv crate doesn't yet support libmpv verions 2.X, I'm using a fork, "libmpv-sirno", which will get deleted once the main one gets updated.  I will need to update this once it that happens. 
