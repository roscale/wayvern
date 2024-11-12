use crate::backends::drm::DrmBackend;
use crate::backends::x11::X11Backend;

pub mod drm;
pub mod x11;

pub enum Backend {
    Drm(DrmBackend),
    X11(X11Backend),
}

impl Backend {
    pub fn drm(&mut self) -> &mut DrmBackend {
        match self {
            Backend::Drm(backend) => backend,
            _ => panic!("Backend is not DRM"),
        }
    }

    pub fn x11(&mut self) -> &mut X11Backend {
        match self {
            Backend::X11(backend) => backend,
            _ => panic!("Backend is not X11"),
        }
    }
}