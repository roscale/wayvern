use std::ops::Deref;

use crate::flutter_engine_rs::flutter_engine_sys::FlutterEngine;

#[derive(Clone, Copy)]
pub struct EngineHandle(pub FlutterEngine);

unsafe impl Send for EngineHandle {}

impl Deref for EngineHandle {
    type Target = FlutterEngine;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
