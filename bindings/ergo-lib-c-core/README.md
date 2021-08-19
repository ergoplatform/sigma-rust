## Core part of the ergo-lib C bindings
Common code that is shared between [iOS/macOS(Swift)](../ergo-lib-ios) and [Android/JVM(JNI/Kotlin)](../ergo-lib-jni) bindings. 
Is responsible for creating and managing pointers for instances of ergo-lib types (`Address`, `ErgoBox`, etc.);
Uses Rust types in it's API and is not suitable for C FFI. See [`ergo-lib-c`](../ergo-lib-c) for ready to use C bindings.
