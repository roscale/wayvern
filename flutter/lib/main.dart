import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_bin/master_layout.dart';
import 'package:wayvern_flutter/wayvern_flutter.dart' as wayvern;

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // await wayvern.setInitialWindowSize(1600, 1200);

  wayvern.init();
  wayvern.initWindowTracker();
  // initFocusManager();

  runWidget(
    ViewCollection(
      views: [
        View(
          view: RendererBinding.instance.platformDispatcher.view(id: 0)!,
          child: MyApp(title: "View 0"),
        ),
      ],
    ),
  );
}

class MyApp extends StatelessWidget {
  final String title;

  const MyApp({super.key, required this.title});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Flutter Demo',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
      ),
      home: MyHomePage(title: title),
    );
  }
}

class MyHomePage extends StatefulWidget {
  final String title;

  const MyHomePage({super.key, required this.title});

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  @override
  Widget build(BuildContext context) {
    return MasterLayout();
  }
}
