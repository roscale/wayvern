use smithay::reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1;
use smithay::utils::{Buffer as BufferCoords, Logical, Point, Rectangle, Size};
use smithay::wayland::compositor;
use smithay::wayland::compositor::{RectangleKind, RegionAttributes};
use smithay::wayland::shell::xdg;

use platform_channels::encodable_value::EncodableValue;

#[derive(Debug)]
pub struct SurfaceCommitMessage {
    pub view_id: u64,
    pub role: Option<&'static str>,
    pub role_state: Option<WlSurfaceRoleState>,
    pub texture_id: i64,
    pub buffer_size: Option<Size<i32, BufferCoords>>,
    pub scale: i32,
    pub input_region: Option<RegionAttributes>,
    pub subsurfaces_below: Vec<u64>,
    pub subsurfaces_above: Vec<u64>,
}

#[derive(Debug)]
pub enum WlSurfaceRoleState {
    Subsurface(SubsurfaceCommitMessage),
    XdgSurface(XdgSurfaceCommitMessage),
}

#[derive(Debug)]
pub struct SubsurfaceCommitMessage {
    pub location: Point<i32, Logical>,
}

#[derive(Debug)]
pub struct XdgSurfaceCommitMessage {
    pub role: Option<&'static str>,
    pub role_state: Option<XdgSurfaceRoleState>,
    pub geometry: Option<Rectangle<i32, Logical>>,
}

#[derive(Debug)]
pub enum XdgSurfaceRoleState {
    Toplevel(XdgToplevelCommitMessage),
    Popup(XdgPopupCommitMessage),
}

#[derive(Debug)]
pub struct XdgToplevelCommitMessage {
    pub decoration: Option<zxdg_toplevel_decoration_v1::Mode>,
}

#[derive(Debug)]
pub struct XdgPopupCommitMessage {
    pub parent_id: u64,
    pub geometry: Rectangle<i32, Logical>,
}

impl SurfaceCommitMessage {
    pub fn serialize(self) -> EncodableValue {
        // TODO: Serialize all the rectangles instead of merging them into one.
        let input_region = if let Some(input_region) = self.input_region {
            let mut acc: Option<Rectangle<i32, Logical>> = None;
            for (kind, rect) in input_region.rects {
                if let RectangleKind::Add = kind {
                    if let Some(acc_) = acc {
                        acc = Some(acc_.merge(rect));
                    } else {
                        acc = Some(rect);
                    }
                }
            }
            acc.unwrap_or_default()
        } else {
            // TODO: Account for DPI scaling.
            self.buffer_size.map(|size| Rectangle::from_loc_and_size((0, 0), (size.w, size.h))).unwrap_or_default()
        };

        let vec = vec![
            (EncodableValue::String("view_id".to_string()), EncodableValue::Int64(self.view_id as i64)),
            (EncodableValue::String("surface".to_string()), EncodableValue::Map(vec![
                (EncodableValue::String("role".to_string()), EncodableValue::Int64(self.role.map(|role| {
                    match role {
                        xdg::XDG_TOPLEVEL_ROLE | xdg::XDG_POPUP_ROLE => 1,
                        compositor::SUBSURFACE_ROLE => 2,
                        _ => 0,
                    }
                }).unwrap_or(0))),
                (EncodableValue::String("role_state".to_string()), self.role_state.map_or(EncodableValue::Null, |role_state| role_state.serialize())),
                (EncodableValue::String("textureId".to_string()), EncodableValue::Int64(self.texture_id)),
                (EncodableValue::String("x".to_string()), EncodableValue::Int32(0)),
                (EncodableValue::String("y".to_string()), EncodableValue::Int32(0)),
                (EncodableValue::String("width".to_string()), EncodableValue::Int32(self.buffer_size.map(|size| size.w).unwrap_or(0))),
                (EncodableValue::String("height".to_string()), EncodableValue::Int32(self.buffer_size.map(|size| size.h).unwrap_or(0))),
                (EncodableValue::String("scale".to_string()), EncodableValue::Int32(self.scale)),
                (EncodableValue::String("subsurfaces_below".to_string()), EncodableValue::List(self.subsurfaces_below.into_iter().map(|id| EncodableValue::Int64(id as i64)).collect())),
                (EncodableValue::String("subsurfaces_above".to_string()), EncodableValue::List(self.subsurfaces_above.into_iter().map(|id| EncodableValue::Int64(id as i64)).collect())),
                (EncodableValue::String("input_region".to_string()), EncodableValue::Map(vec![
                    (EncodableValue::String("x1".to_string()), EncodableValue::Int32(input_region.loc.x)),
                    (EncodableValue::String("y1".to_string()), EncodableValue::Int32(input_region.loc.y)),
                    (EncodableValue::String("x2".to_string()), EncodableValue::Int32(input_region.loc.x + input_region.size.w)),
                    (EncodableValue::String("y2".to_string()), EncodableValue::Int32(input_region.loc.y + input_region.size.h)),
                ])),
            ])),
        ];

        EncodableValue::Map(vec)
    }
}

impl WlSurfaceRoleState {
    pub fn serialize(self) -> EncodableValue {
        match self {
            WlSurfaceRoleState::Subsurface(subsurface) => subsurface.serialize(),
            WlSurfaceRoleState::XdgSurface(xdg_surface) => xdg_surface.serialize(),
        }
    }
}

impl SubsurfaceCommitMessage {
    pub fn serialize(self) -> EncodableValue {
        EncodableValue::Map(vec![
            (EncodableValue::String("x".to_string()), EncodableValue::Int64(self.location.x as i64)),
            (EncodableValue::String("y".to_string()), EncodableValue::Int64(self.location.y as i64)),
        ])
    }
}

impl XdgSurfaceCommitMessage {
    pub fn serialize(self) -> EncodableValue {
        EncodableValue::Map(vec![
            (EncodableValue::String("role".to_string()), EncodableValue::Int64(self.role.map(|role| {
                match role {
                    xdg::XDG_TOPLEVEL_ROLE => 1,
                    xdg::XDG_POPUP_ROLE => 2,
                    _ => 0,
                }
            }).unwrap_or(0))),
            (EncodableValue::String("role_state".to_string()), self.role_state.map_or(EncodableValue::Null, |role_state| role_state.serialize())),
            (EncodableValue::String("x".to_string()), EncodableValue::Int64(self.geometry.map(|geometry| geometry.loc.x).unwrap_or(0) as i64)),
            (EncodableValue::String("y".to_string()), EncodableValue::Int64(self.geometry.map(|geometry| geometry.loc.y).unwrap_or(0) as i64)),
            (EncodableValue::String("width".to_string()), EncodableValue::Int64(self.geometry.map(|geometry| geometry.size.w).unwrap_or(0) as i64)),
            (EncodableValue::String("height".to_string()), EncodableValue::Int64(self.geometry.map(|geometry| geometry.size.h).unwrap_or(0) as i64)),
        ])
    }
}

impl XdgSurfaceRoleState {
    pub fn serialize(self) -> EncodableValue {
        match self {
            XdgSurfaceRoleState::Toplevel(toplevel) => toplevel.serialize(),
            XdgSurfaceRoleState::Popup(popup) => popup.serialize(),
        }
    }
}

impl XdgToplevelCommitMessage {
    pub fn serialize(self) -> EncodableValue {
        EncodableValue::Map(vec![
            (EncodableValue::String("decoration".to_string()), self.decoration.map_or(EncodableValue::Null, |v| EncodableValue::Int64(v as i64))),
        ])
    }
}

impl XdgPopupCommitMessage {
    pub fn serialize(self) -> EncodableValue {
        EncodableValue::Map(vec![
            (EncodableValue::String("parent_id".to_string()), EncodableValue::Int64(self.parent_id as i64)),
            (EncodableValue::String("x".to_string()), EncodableValue::Int64(self.geometry.loc.x as i64)),
            (EncodableValue::String("y".to_string()), EncodableValue::Int64(self.geometry.loc.y as i64)),
            (EncodableValue::String("width".to_string()), EncodableValue::Int64(self.geometry.size.w as i64)),
            (EncodableValue::String("height".to_string()), EncodableValue::Int64(self.geometry.size.h as i64)),
        ])
    }
}
