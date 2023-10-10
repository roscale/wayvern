use std::ffi::c_void;

use tracing::error;

use crate::flutter_engine::{Baton, FlutterEngineData};
use crate::flutter_engine::embedder::{FlutterPresentInfo, FlutterTransformation};

pub unsafe extern "C" fn make_current(user_data: *mut c_void) -> bool {
    let state = &mut *(user_data as *mut FlutterEngineData);
    match state.main_egl_context.make_current() {
        Ok(()) => true,
        Err(err) => {
            error!("{}", err);
            false
        },
    }
}

pub unsafe extern "C" fn make_resource_current(user_data: *mut c_void) -> bool {
    let state = &mut *(user_data as *mut FlutterEngineData);
    match state.resource_egl_context.make_current() {
        Ok(()) => true,
        Err(err) => {
            error!("{}", err);
            false
        },
    }
}

pub unsafe extern "C" fn clear_current(user_data: *mut c_void) -> bool {
    let state = &mut *(user_data as *mut FlutterEngineData);
    match state.main_egl_context.unbind() {
        Ok(()) => true,
        Err(err) => {
            error!("{}", err);
            false
        },
    }
}

pub unsafe extern "C" fn fbo_callback(user_data: *mut c_void) -> u32 {
    let state = &mut *(user_data as *mut FlutterEngineData);
    if state.channels.tx_request_rbo.send(()).is_err() {
        return 0;
    }
    if let Ok(Some(dmabuf)) = state.channels.rx_rbo.recv() {
        state.framebuffer_importer.import_framebuffer(&state.main_egl_context, dmabuf).unwrap_or(0)
    } else {
        0
    }
}

pub unsafe extern "C" fn present_with_info(user_data: *mut ::std::os::raw::c_void, _frame_present_info: *const FlutterPresentInfo) -> bool {
    let state = &mut *(user_data as *mut FlutterEngineData);
    state.gl.Finish();
    state.channels.tx_present.send(()).is_ok()
}

pub unsafe extern "C" fn surface_transformation(user_data: *mut ::std::os::raw::c_void) -> FlutterTransformation {
    let state = &mut *(user_data as *mut FlutterEngineData);

    while let Ok(output_height) = state.channels.rx_output_height.try_recv() {
        state.output_height = Some(output_height);
    }

    match state.output_height {
        Some(output_height) => FlutterTransformation {
            scaleX: 1.0,
            skewX: 0.0,
            transX: 0.0,
            skewY: 0.0,
            scaleY: -1.0,
            transY: output_height as f64,
            pers0: 0.0,
            pers1: 0.0,
            pers2: 1.0,
        },
        None => FlutterTransformation {
            scaleX: 1.0,
            skewX: 0.0,
            transX: 0.0,
            skewY: 0.0,
            scaleY: 1.0,
            transY: 0.0,
            pers0: 0.0,
            pers1: 0.0,
            pers2: 1.0,
        },
    }
}

pub unsafe extern "C" fn _vsync_callback(user_data: *mut std::os::raw::c_void, baton: isize) {
    let state = &mut *(user_data as *mut FlutterEngineData);
    let _ = state.channels.tx_baton.send(Baton(baton));
}

pub enum FlutterEngineIntent {
    RequestFramebuffer,
    PresentFramebuffer,
}

impl TryFrom<u32> for FlutterEngineIntent {
    type Error = ();

    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            x if x == FlutterEngineIntent::RequestFramebuffer as u32 => Ok(FlutterEngineIntent::RequestFramebuffer),
            x if x == FlutterEngineIntent::PresentFramebuffer as u32 => Ok(FlutterEngineIntent::PresentFramebuffer),
            _ => Err(()),
        }
    }
}
