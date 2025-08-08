import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:wayvern_flutter/wayvern_flutter.dart' as wayvern;

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // await wayvern.setInitialWindowSize(1600, 1200);

  wayvern.init();

  wayvern.windowCreated.listen((id) => print("Window $id created"));
  wayvern.windowDestroyed.listen((id) => print("Window $id destroyed"));

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
  int _counter = 0;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
        title: Text(widget.title),
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: <Widget>[
            const Text('hehehei have pushed the button this many times:'),
            Text(
              '$_counter',
              style: Theme.of(context).textTheme.headlineMedium,
            ),
            CircularProgressIndicator(),
            Padding(padding: EdgeInsetsGeometry.only(top: 50)),

            ListenableBuilder(
              listenable: wayvern.windows,
              builder: (_, _) {
                return Row(
                  mainAxisSize: MainAxisSize.min,
                  children:
                      wayvern.windows.list.map((window) {
                        return ListenableBuilder(
                          listenable: window,
                          builder: (_, _) {
                            return SizedBox(
                              width: window.width.toDouble(),
                              height: window.height.toDouble(),
                              child: Texture(textureId: window.textureId),
                            );
                          },
                        );
                      }).toList(),
                );
              },
            ),
          ],
        ),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _incrementCounter,
        tooltip: 'Increment',
        child: const Icon(Icons.add),
      ),
    );
  }

  void _incrementCounter() {
    setState(() {
      _counter++;
    });
  }
}
