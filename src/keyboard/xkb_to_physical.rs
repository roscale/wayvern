//! Maps XKB specific key code values representing [PhysicalKeyboardKey].

use std::collections::HashMap;
use lazy_static::lazy_static;
use crate::keyboard::physical_keys::*;


lazy_static! {
    pub static ref xkb_to_physical_map: HashMap<u32, u64> = {
        HashMap::from([
            (0x00000009, escape),
            (0x0000000a, digit1),
            (0x0000000b, digit2),
            (0x0000000c, digit3),
            (0x0000000d, digit4),
            (0x0000000e, digit5),
            (0x0000000f, digit6),
            (0x00000010, digit7),
            (0x00000011, digit8),
            (0x00000012, digit9),
            (0x00000013, digit0),
            (0x00000014, minus),
            (0x00000015, equal),
            (0x00000016, backspace),
            (0x00000017, tab),
            (0x00000018, keyQ),
            (0x00000019, keyW),
            (0x0000001a, keyE),
            (0x0000001b, keyR),
            (0x0000001c, keyT),
            (0x0000001d, keyY),
            (0x0000001e, keyU),
            (0x0000001f, keyI),
            (0x00000020, keyO),
            (0x00000021, keyP),
            (0x00000022, bracketLeft),
            (0x00000023, bracketRight),
            (0x00000024, enter),
            (0x00000025, controlLeft),
            (0x00000026, keyA),
            (0x00000027, keyS),
            (0x00000028, keyD),
            (0x00000029, keyF),
            (0x0000002a, keyG),
            (0x0000002b, keyH),
            (0x0000002c, keyJ),
            (0x0000002d, keyK),
            (0x0000002e, keyL),
            (0x0000002f, semicolon),
            (0x00000030, quote),
            (0x00000031, backquote),
            (0x00000032, shiftLeft),
            (0x00000033, backslash),
            (0x00000034, keyZ),
            (0x00000035, keyX),
            (0x00000036, keyC),
            (0x00000037, keyV),
            (0x00000038, keyB),
            (0x00000039, keyN),
            (0x0000003a, keyM),
            (0x0000003b, comma),
            (0x0000003c, period),
            (0x0000003d, slash),
            (0x0000003e, shiftRight),
            (0x0000003f, numpadMultiply),
            (0x00000040, altLeft),
            (0x00000041, space),
            (0x00000042, capsLock),
            (0x00000043, f1),
            (0x00000044, f2),
            (0x00000045, f3),
            (0x00000046, f4),
            (0x00000047, f5),
            (0x00000048, f6),
            (0x00000049, f7),
            (0x0000004a, f8),
            (0x0000004b, f9),
            (0x0000004c, f10),
            (0x0000004d, numLock),
            (0x0000004e, scrollLock),
            (0x0000004f, numpad7),
            (0x00000050, numpad8),
            (0x00000051, numpad9),
            (0x00000052, numpadSubtract),
            (0x00000053, numpad4),
            (0x00000054, numpad5),
            (0x00000055, numpad6),
            (0x00000056, numpadAdd),
            (0x00000057, numpad1),
            (0x00000058, numpad2),
            (0x00000059, numpad3),
            (0x0000005a, numpad0),
            (0x0000005b, numpadDecimal),
            (0x0000005d, lang5),
            (0x0000005e, intlBackslash),
            (0x0000005f, f11),
            (0x00000060, f12),
            (0x00000061, intlRo),
            (0x00000062, lang3),
            (0x00000063, lang4),
            (0x00000064, convert),
            (0x00000065, kanaMode),
            (0x00000066, nonConvert),
            (0x00000068, numpadEnter),
            (0x00000069, controlRight),
            (0x0000006a, numpadDivide),
            (0x0000006b, printScreen),
            (0x0000006c, altRight),
            (0x0000006e, home),
            (0x0000006f, arrowUp),
            (0x00000070, pageUp),
            (0x00000071, arrowLeft),
            (0x00000072, arrowRight),
            (0x00000073, end),
            (0x00000074, arrowDown),
            (0x00000075, pageDown),
            (0x00000076, insert),
            (0x00000077, delete),
            (0x00000079, audioVolumeMute),
            (0x0000007a, audioVolumeDown),
            (0x0000007b, audioVolumeUp),
            (0x0000007c, power),
            (0x0000007d, numpadEqual),
            (0x0000007e, numpadSignChange),
            (0x0000007f, pause),
            (0x00000080, showAllWindows),
            (0x00000081, numpadComma),
            (0x00000082, lang1),
            (0x00000083, lang2),
            (0x00000084, intlYen),
            (0x00000085, metaLeft),
            (0x00000086, metaRight),
            (0x00000087, contextMenu),
            (0x00000088, browserStop),
            (0x00000089, again),
            (0x0000008b, undo),
            (0x0000008c, select),
            (0x0000008d, copy),
            (0x0000008e, open),
            (0x0000008f, paste),
            (0x00000090, find),
            (0x00000091, cut),
            (0x00000092, help),
            (0x00000094, launchApp2),
            (0x00000096, sleep),
            (0x00000097, wakeUp),
            (0x00000098, launchApp1),
            (0x0000009e, launchInternetBrowser),
            (0x000000a0, lockScreen),
            (0x000000a3, launchMail),
            (0x000000a4, browserFavorites),
            (0x000000a6, browserBack),
            (0x000000a7, browserForward),
            (0x000000a9, eject),
            (0x000000ab, mediaTrackNext),
            (0x000000ac, mediaPlayPause),
            (0x000000ad, mediaTrackPrevious),
            (0x000000ae, mediaStop),
            (0x000000af, mediaRecord),
            (0x000000b0, mediaRewind),
            (0x000000b1, launchPhone),
            (0x000000b3, mediaSelect),
            (0x000000b4, browserHome),
            (0x000000b5, browserRefresh),
            (0x000000b6, exit),
            (0x000000bb, numpadParenLeft),
            (0x000000bc, numpadParenRight),
            (0x000000bd, newKey),
            (0x000000be, redo),
            (0x000000bf, f13),
            (0x000000c0, f14),
            (0x000000c1, f15),
            (0x000000c2, f16),
            (0x000000c3, f17),
            (0x000000c4, f18),
            (0x000000c5, f19),
            (0x000000c6, f20),
            (0x000000c7, f21),
            (0x000000c8, f22),
            (0x000000c9, f23),
            (0x000000ca, f24),
            (0x000000d1, mediaPause),
            (0x000000d6, close),
            (0x000000d7, mediaPlay),
            (0x000000d8, mediaFastForward),
            (0x000000d9, bassBoost),
            (0x000000da, print),
            (0x000000e1, browserSearch),
            (0x000000e8, brightnessDown),
            (0x000000e9, brightnessUp),
            (0x000000eb, displayToggleIntExt),
            (0x000000ed, kbdIllumDown),
            (0x000000ee, kbdIllumUp),
            (0x000000ef, mailSend),
            (0x000000f0, mailReply),
            (0x000000f1, mailForward),
            (0x000000f2, save),
            (0x000000f3, launchDocuments),
            (0x000000fc, brightnessAuto),
            (0x00000100, microphoneMuteToggle),
            (0x0000016e, info),
            (0x00000172, programGuide),
            (0x0000017a, closedCaptionToggle),
            (0x0000017c, zoomToggle),
            (0x0000017e, launchKeyboardLayout),
            (0x00000190, launchAudioBrowser),
            (0x00000195, launchCalendar),
            (0x0000019d, mediaLast),
            (0x000001a2, channelUp),
            (0x000001a3, channelDown),
            (0x000001aa, zoomIn),
            (0x000001ab, zoomOut),
            (0x000001ad, launchWordProcessor),
            (0x000001af, launchSpreadsheet),
            (0x000001b5, launchContacts),
            (0x000001b7, brightnessToggle),
            (0x000001b8, spellCheck),
            (0x000001b9, logOff),
            (0x0000024b, launchControlPanel),
            (0x0000024c, selectTask),
            (0x0000024d, launchScreenSaver),
            (0x0000024e, speechInputToggle),
            (0x0000024f, launchAssistant),
            (0x00000250, keyboardLayoutSelect),
            (0x00000258, brightnessMinimum),
            (0x00000259, brightnessMaximum),
            (0x00000281, privacyScreenToggle),
        ])
    };
}


























































































































































































































