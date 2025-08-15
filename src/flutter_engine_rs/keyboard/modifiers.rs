pub struct FlutterModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub logo: bool,
    pub caps_lock: bool,
    pub num_lock: bool,
}

pub fn get_modifiers_bitfield(mods: FlutterModifiers) -> u32 {
    let mut mask = 0;

    if mods.shift {
        mask |= 1 << 0;
    }
    if mods.caps_lock {
        mask |= 1 << 1;
    }
    if mods.ctrl {
        mask |= 1 << 2;
    }
    if mods.alt {
        mask |= 1 << 3;
    }
    if mods.num_lock {
        mask |= 1 << 4;
    }
    if mods.logo {
        mask |= 1 << 26;
    }

    mask
}
