//! Utilities

use elliptic_curve::subtle::CtOption;

/// Convert to Option<T>
pub trait IntoOption<T> {
    /// Get Option<T>
    fn into_option(self) -> Option<T>;
}

impl<T> IntoOption<T> for CtOption<T> {
    fn into_option(self) -> Option<T> {
        if self.is_some().into() {
            Some(self.unwrap())
        } else {
            None
        }
    }
}
