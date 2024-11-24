import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_portal/flutter_portal.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:zenith/ui/common/state/tasks_provider.dart';
import 'package:zenith/ui/common/state/wayland_state.dart';
import 'package:zenith/ui/common/state/xdg_toplevel_state.dart';
import 'package:zenith/ui/desktop/app_drawer/app_drawer_button.dart';
import 'package:zenith/ui/desktop/state/task_switcher_provider.dart';
import 'package:zenith/ui/desktop/task_bar.dart';
import 'package:zenith/ui/desktop/task_switcher.dart';
import 'package:zenith/ui/desktop/window_manager.dart';

class DesktopUi extends ConsumerStatefulWidget {
  const DesktopUi({Key? key}) : super(key: key);

  @override
  ConsumerState<DesktopUi> createState() => _DesktopUiState();
}

class SingleLogicalKeyReleaseActivator extends ShortcutActivator {
  LogicalKeyboardKey key;

  SingleLogicalKeyReleaseActivator(this.key);

  @override
  bool accepts(KeyEvent event, HardwareKeyboard state) {
    if (event is! KeyUpEvent) {
      return false;
    }
    return event.logicalKey == key;
  }

  @override
  String debugDescribeKeys() => "";

  @override
  Iterable<LogicalKeyboardKey>? get triggers => [key];
}

class ToggleAppDrawerIntent extends Intent {}

class ShowTaskSwitcher extends Intent {
  /// If false, it goes to previous.
  final bool goToNext;

  const ShowTaskSwitcher(this.goToNext);
}

class HideTaskSwitcher extends Intent {}

class TaskSwitcherGoToNext extends Intent {}

class TaskSwitcherGoToPrevious extends Intent {}

class _DesktopUiState extends ConsumerState<DesktopUi> {
  final backgroundFocusNode = FocusNode();

  @override
  Widget build(BuildContext context) {
    return Consumer(
      builder: (BuildContext context, WidgetRef ref, Widget? child) {
        bool taskSwitcherShown =
            ref.watch(taskSwitcherProvider.select((value) => value.shown));
        final taskSwitcherShortcuts = !taskSwitcherShown
            ? {
                const SingleActivator(
                  LogicalKeyboardKey.tab,
                  meta: true,
                  includeRepeats: false,
                ): const ShowTaskSwitcher(true),
                //
                const SingleActivator(
                  LogicalKeyboardKey.tab,
                  meta: true,
                  shift: true,
                  includeRepeats: false,
                ): const ShowTaskSwitcher(false),
              }
            : {
                SingleLogicalKeyReleaseActivator(
                  LogicalKeyboardKey.metaLeft,
                ): HideTaskSwitcher(),
                //
                const SingleActivator(
                  LogicalKeyboardKey.tab,
                  meta: true,
                  includeRepeats: false,
                ): TaskSwitcherGoToNext(),
                //
                const SingleActivator(
                  LogicalKeyboardKey.tab,
                  meta: true,
                  shift: true,
                  includeRepeats: false,
                ): TaskSwitcherGoToPrevious(),
              };

        return Shortcuts(
          shortcuts: {
            SingleLogicalKeyReleaseActivator(LogicalKeyboardKey.metaLeft):
                ToggleAppDrawerIntent(),
            ...taskSwitcherShortcuts,
          },
          child: child!,
        );
      },
      child: Actions(
        actions: {
          ToggleAppDrawerIntent: CallbackAction(onInvoke: (_) {
            ref
                .read(appDrawerVisibleProvider.notifier)
                .update((visible) => !visible);
            return null;
          }),
          ShowTaskSwitcher:
              CallbackAction<ShowTaskSwitcher>(onInvoke: (intent) {
            ref.read(taskSwitcherProvider.notifier).show(intent.goToNext);
            return null;
          }),
          HideTaskSwitcher: CallbackAction(onInvoke: (_) {
            ref.read(taskSwitcherProvider.notifier).hide();
            final tasks = ref.read(tasksProvider).tasks;
            if (tasks.isNotEmpty) {
              int len = tasks.length;
              // Because the task switcher shows the tasks in reverse order.
              // The first one is the most recent.
              int index = len - ref.read(taskSwitcherProvider).index - 1;
              int viewId = tasks[index];
              ref.read(tasksProvider.notifier).putInFront(viewId);
              ref
                  .read(waylandProviderProvider)
                  .xdgToplevels[viewId]!
                  .focusNode
                  .requestFocus();
            }
            return null;
          }),
          TaskSwitcherGoToNext: CallbackAction(onInvoke: (_) {
            ref.read(taskSwitcherProvider.notifier).next();
            return null;
          }),
          TaskSwitcherGoToPrevious: CallbackAction(onInvoke: (_) {
            ref.read(taskSwitcherProvider.notifier).previous();
            return null;
          }),
        },
        child: FocusScope(
          autofocus: true,
          child: Portal(
            child: Stack(
              clipBehavior: Clip.none,
              fit: StackFit.expand,
              children: [
                Focus(
                  focusNode: backgroundFocusNode,
                  child: GestureDetector(
                    onTapDown: (_) {
                      backgroundFocusNode.requestFocus();
                    },
                    child: Image.asset("assets/images/background.jpg",
                        fit: BoxFit.cover),
                  ),
                ),
                Overlay(
                  initialEntries: [
                    OverlayEntry(
                      builder: (_) => const Column(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Expanded(
                            child: WindowManager(),
                          ),
                          TaskBar(),
                        ],
                      ),
                    ),
                  ],
                ),
                Consumer(
                  builder:
                      (BuildContext context, WidgetRef ref, Widget? child) {
                    bool shown = ref.watch(
                        taskSwitcherProvider.select((value) => value.shown));
                    return shown ? child! : const SizedBox();
                  },
                  child: const Center(
                    child: TaskSwitcherWidget(),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  @override
  void dispose() {
    backgroundFocusNode.dispose();
    super.dispose();
  }
}
