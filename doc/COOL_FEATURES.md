# Cool features
I like showing off.

## Recompiles when necessary
`foof` will not recompile your code if the source file is untouched.
It stores a SHA hash of the source in the temp folder, and will not re-build if they match.

## Caches crates.io endpoints
Don't worry about hammering crates, as `foof` will remember what version `latest` actually is!
If it changes, `foof` will re-fetch on the next run once an hour has passed, or immediately with the `--force-recache` flag.

## Quiet mode
Your beloved `cargo run -q` can be accessed with `foof -q`. This will also mute its own console output, leaving you with nothing between you and your code.

## `chumsky`-based directive parser
I really, really like this library. If you need to make a parser-combinator for any reason, do check it out. It's so beautiful to use.