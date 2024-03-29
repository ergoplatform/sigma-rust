import Foundation
import ErgoLibC


private class WrapClosure<T> {
    fileprivate let closure: T
    init(closure: T) {
        self.closure = closure
    }
}

private func wrapClosureRawPtr(_ closure: @escaping (Result<UnsafeRawPointer, Error>) -> Void) -> CompletionCallback {
    // base on https://www.nickwilcox.com/blog/recipe_swift_rust_callback/
    // step 1: manually increment reference count on closure
    let wrappedClosure = WrapClosure(closure: closure)
    let userdata = Unmanaged.passRetained(wrappedClosure).toOpaque()

    // step 2: create C compatible function pointer
    let callback: @convention(c) (UnsafeMutableRawPointer, UnsafeRawPointer?, ErrorPtr?) -> Void = {
        (_ userdata: UnsafeMutableRawPointer, _ resPtr: UnsafeRawPointer?,  _ errorPtr: ErrorPtr?) in
        // reverse step 1 and manually decrement reference count on the closure and turn it back to Swift type.
        // Because we are back to letting Swift manage our reference count, when the scope ends the wrapped closure will be freed.
        let wrappedClosure: WrapClosure<(Result<UnsafeRawPointer, Error>) -> Void> =
            Unmanaged.fromOpaque(userdata).takeRetainedValue()

        if let resPtr = resPtr {
            // success
            let res = Result<UnsafeRawPointer, Error>.success(resPtr)
            DispatchQueue.main.async {
                wrappedClosure.closure(res)
            }
        } else {
            // failure
            let cStringReason = ergo_lib_error_to_string(errorPtr!)
            let reason = String(cString: cStringReason!)
            ergo_lib_delete_string(cStringReason)
            ergo_lib_delete_error(errorPtr)
            let res = Result<UnsafeRawPointer, Error>.failure(RestNodeApiError.misc(reason))
            DispatchQueue.main.async {
                wrappedClosure.closure(res)
            }
        }
    }

    // called on task cancellation (abort)
    let abortCallback: @convention(c) (UnsafeMutableRawPointer) -> Void = { 
        (_ userdata: UnsafeMutableRawPointer) in
        // reverse step 1 and manually decrement reference count on the closure and turn it back to Swift type.
        // Because we are back to letting Swift manage our reference count, when the scope ends 
        // the wrapped closure will be freed.
        let c_ptr: Unmanaged<WrapClosure<(Result<UnsafeRawPointer, Error>) -> Void>> = Unmanaged.fromOpaque(userdata)
        _ = c_ptr.takeRetainedValue()
    }

    return CompletionCallback( user_data: userdata, completion_callback: callback, abort_callback: abortCallback)
}

/// Wraps closure into the struct with C compatible function pointers and memory management
internal func wrapClosure<T: FromRawPtr>(_ closure: @escaping (Result<T, Error>) -> Void) -> CompletionCallback {
        let closure: (Result<UnsafeRawPointer, Error>) -> Void = {
            (res: Result<UnsafeRawPointer, Error>) in
            let mapped = res.map { rawPtr in 
                T.fromRawPtr(ptr: rawPtr)
            }
            closure(mapped)
        }
        // We have to go through UnsafeRawPointer because generic types cannot be captured 
        // inside @convention(c) closure
        return wrapClosureRawPtr(closure)
}

