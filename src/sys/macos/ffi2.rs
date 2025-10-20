use std::ops::Deref;

use objc2_core_foundation::{CFRetained, Type};

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct CFRetainedSafe<T: Type>(pub CFRetained<T>);

impl<T: Type> Clone for CFRetainedSafe<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

unsafe impl<T: Type> Send for CFRetainedSafe<T> {}

impl<T: Type> Deref for CFRetainedSafe<T> {
    type Target = CFRetained<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
