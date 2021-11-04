## Swift wrapper for [C bindings](../ergo-lib-c) of ergo-lib.


## Build instructions (draft)

First build `ergo-lib-c`
```
cargo build -p ergo-lib-c
```

This creates a static library under `<project_root_directory>/target/debug`.
To build this project we need to point `swift` to this directory for linking.
```
swift build -Xlinker -L../../../target/debug/
```

To run tests we must also pass in the library directory:
```
swift test -Xlinker -L../../../target/debug/
```
 
