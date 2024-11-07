pub mod drm;
pub mod x11;

pub trait Backend {
    fn seat_name(&self) -> String;
}