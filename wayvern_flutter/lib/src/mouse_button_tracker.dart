class MouseButtonTracker {
  Map<int, int> buttons = {};

  List<Button> trackButtons(int device, int buttons) {
    final oldButtons = this.buttons[device] ?? 0;
    final newButtons = buttons;

    if (oldButtons == newButtons) {
      return [];
    }

    List<Button> list = [];

    for (int i = 0; i < 64; i++) {
      int mask = 1 << i;

      bool pressedBefore = (oldButtons & mask) != 0;
      bool pressedNow = (newButtons & mask) != 0;

      bool pressed;
      if (pressedNow && !pressedBefore) {
        pressed = true;
      } else if (!pressedNow && pressedBefore) {
        pressed = false;
      } else {
        continue;
      }

      list.add(Button(mask, pressed));
    }

    this.buttons[device] = buttons;

    return list;
  }
}

class Button {
  final int button;
  final bool pressed;

  const Button(this.button, this.pressed);
}
