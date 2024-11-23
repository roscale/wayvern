use std::collections::HashMap;
use lazy_static::lazy_static;
use crate::keyboard::logical_keys::*;

lazy_static! {
    pub static ref glfw_to_logical_map: HashMap<u32, u64> = {
        HashMap::from([
            (32, space),
            (39, quote),
            (44, comma),
            (45, minus),
            (46, period),
            (47, slash),
            (48, digit0),
            (49, digit1),
            (50, digit2),
            (51, digit3),
            (52, digit4),
            (53, digit5),
            (54, digit6),
            (55, digit7),
            (56, digit8),
            (57, digit9),
            (59, semicolon),
            (61, equal),
            (65, keyA),
            (66, keyB),
            (67, keyC),
            (68, keyD),
            (69, keyE),
            (70, keyF),
            (71, keyG),
            (72, keyH),
            (73, keyI),
            (74, keyJ),
            (75, keyK),
            (76, keyL),
            (77, keyM),
            (78, keyN),
            (79, keyO),
            (80, keyP),
            (81, keyQ),
            (82, keyR),
            (83, keyS),
            (84, keyT),
            (85, keyU),
            (86, keyV),
            (87, keyW),
            (88, keyX),
            (89, keyY),
            (90, keyZ),
            (91, bracketLeft),
            (92, backslash),
            (93, bracketRight),
            (96, backquote),
            (256, escape),
            (257, enter),
            (258, tab),
            (259, backspace),
            (260, insert),
            (261, delete),
            (262, arrowRight),
            (263, arrowLeft),
            (264, arrowDown),
            (265, arrowUp),
            (266, pageUp),
            (267, pageDown),
            (268, home),
            (269, end),
            (280, capsLock),
            (282, numLock),
            (283, printScreen),
            (284, pause),
            (290, f1),
            (291, f2),
            (292, f3),
            (293, f4),
            (294, f5),
            (295, f6),
            (296, f7),
            (297, f8),
            (298, f9),
            (299, f10),
            (300, f11),
            (301, f12),
            (302, f13),
            (303, f14),
            (304, f15),
            (305, f16),
            (306, f17),
            (307, f18),
            (308, f19),
            (309, f20),
            (310, f21),
            (311, f22),
            (312, f23),
            (320, numpad0),
            (321, numpad1),
            (322, numpad2),
            (323, numpad3),
            (324, numpad4),
            (325, numpad5),
            (326, numpad6),
            (327, numpad7),
            (328, numpad8),
            (329, numpad9),
            (330, numpadDecimal),
            (331, numpadDivide),
            (332, numpadMultiply),
            (334, numpadAdd),
            (335, numpadEnter),
            (336, numpadEqual),
            (340, shiftLeft),
            (341, controlLeft),
            (342, altLeft),
            (343, metaLeft),
            (344, shiftRight),
            (345, controlRight),
            (346, altRight),
            (347, metaRight),
            (348, contextMenu),
        ])
    };
}