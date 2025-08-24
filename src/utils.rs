use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};

use smithay::desktop::{Space, Window};
use tracing::error;

use crate::{
    flutter_engine_rs::platform_channels::method_result::MethodResult,
    wayland::handlers::xdg::MyWindowData,
};

/// An [Option] that automatically dereferences into the wrapping type.
/// Panics if dereferenced when None.
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

pub trait GetSurfaceId {
    fn get_surface_id(&self) -> usize;
}

impl GetSurfaceId for Window {
    fn get_surface_id(&self) -> usize {
        self.user_data()
            .get::<RefCell<MyWindowData>>()
            .unwrap()
            .borrow()
            .surface_id
    }
}

pub trait FindSurfaceId {
    fn find_surface_id(&self, surface_id: usize) -> Option<Window>;
}

impl FindSurfaceId for Space<Window> {
    fn find_surface_id(&self, surface_id: usize) -> Option<Window> {
        self.elements()
            .find(|window| window.get_surface_id() == surface_id)
            .cloned()
    }
}

pub fn window_not_found(surface_id: usize, result: &mut Box<dyn MethodResult + 'static>) {
    let message = format!("Window with surface id {surface_id} doesn't exist");
    error!(message);
    result.error("error".to_string(), message, None);
}
