// Copyright 2018 The Exonum Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use failure::Error;
use jni::JNIEnv;
use std::any::Any;
use std::thread;

type ExceptionResult<T> = thread::Result<Result<T, Error>>;

// Returns value or "throws" exception. `error_val` is returned, because exception will be thrown
// at the Java side. So this function should be used only for the `panic::catch_unwind` result.
pub fn unwrap_exc_or<T>(env: &JNIEnv, res: ExceptionResult<T>, error_val: T) -> T {
    match res {
        Ok(val) => {
            match val {
                Ok(val) => val,
                Err(jni_error) => {
                    // Do nothing if there is a pending Java-exception that will be thrown
                    // automatically by the JVM when the native method returns.
                    if !env.exception_check().unwrap() {
                        // Throw a Java exception manually in case of an internal error.
                        throw(env, &jni_error.to_string())
                    }
                    error_val
                }
            }
        }
        Err(ref e) => {
            throw(env, &any_to_string(e));
            error_val
        }
    }
}

// Same as `unwrap_exc_or` but returns default value.
#[allow(dead_code)]
pub fn unwrap_exc_or_default<T: Default>(env: &JNIEnv, res: ExceptionResult<T>) -> T {
    unwrap_exc_or(env, res, T::default())
}

// Calls a corresponding `JNIEnv` method, so exception will be thrown when execution returns to
// the Java side.
fn throw(env: &JNIEnv, description: &str) {
    // We cannot throw exception from this function, so errors should be written in log instead.
    let exception = match env.find_class("java/lang/RuntimeException") {
        Ok(val) => val,
        Err(e) => {
            error!(
                "Unable to find 'RuntimeException' class: {}",
                e.description()
            );
            return;
        }
    };
    if let Err(e) = env.throw_new(exception, description) {
        error!(
            "Unable to find 'RuntimeException' class: {}",
            e.description()
        );
    }
}

// Tries to get meaningful description from panic-error.
pub fn any_to_string(any: &Box<dyn Any + Send>) -> String {
    if let Some(s) = any.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = any.downcast_ref::<String>() {
        s.clone()
    } else if let Some(error) = any.downcast_ref::<Box<dyn std::error::Error + Send>>() {
        error.to_string()
    } else {
        "Unknown error occurred".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::panic;

    #[test]
    fn str_any() {
        let string = "Static string (&str)";
        let error = panic_error(string);
        assert_eq!(string, any_to_string(&error));
    }

    #[test]
    fn string_any() {
        let string = "Owned string (String)".to_owned();
        let error = panic_error(string.clone());
        assert_eq!(string, any_to_string(&error));
    }

    #[test]
    fn box_error_any() {
        let error: Box<dyn Error + Send> = Box::new("e".parse::<i32>().unwrap_err());
        let description = error.to_string();
        let error = panic_error(error);
        assert_eq!(description, any_to_string(&error));
    }

    #[test]
    fn unknown_any() {
        let error = panic_error(1);
        assert_eq!("Unknown error occurred", any_to_string(&error));
    }

    fn panic_error<T: Send + 'static>(val: T) -> Box<dyn Any + Send> {
        panic::catch_unwind(panic::AssertUnwindSafe(|| panic!(val))).unwrap_err()
    }
}
