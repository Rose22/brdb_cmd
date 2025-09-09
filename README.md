lets you read (and in the future, edit) files inside a brickadia brdb world file

## how to use
you need rust installed first. then,

to install:
```
cargo build --release
cp target/release/brdb_cmd /wherever/you/want
```

then to use:
```
/path/to/brdb_cmd /path/to/world.brdb ls|read|edit /path/to/file
```
