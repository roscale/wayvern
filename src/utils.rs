use std::ops::{Deref, DerefMut};

pub struct AutoDerefOption<T>(pub Option<T>);

impl<T> Deref for AutoDerefOption<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect("AutoDerefOption is None")
    }
}

impl<T> DerefMut for AutoDerefOption<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().expect("AutoDerefOption is None")
    }
}

impl<T> AsRef<Option<T>> for AutoDerefOption<T> {
    fn as_ref(&self) -> &Option<T> {
        &self.0
    }
}

impl<T> AsMut<Option<T>> for AutoDerefOption<T> {
    fn as_mut(&mut self) -> &mut Option<T> {
        &mut self.0
    }
}

impl<T> From<Option<T>> for AutoDerefOption<T> {
    fn from(value: Option<T>) -> Self {
        Self(value)
    }
}

