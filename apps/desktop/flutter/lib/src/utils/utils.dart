import 'dart:io';

import 'package:path/path.dart' as path;
import 'package:path_provider/path_provider.dart';

import 'env.dart';
import 'platform_util.dart';

// 请按文件名排序放置

final sharedEnv = Env.instance;

Directory? _dataDirectory;

Future<Directory> getAppDirectory() async {
  if (_dataDirectory == null) {
    final docDir = await getApplicationDocumentsDirectory();

    if (kIsLinux || kIsWindows) {
      _dataDirectory = Directory(
        path.join(docDir.parent.path, '.linguaray', 'v2'),
      );
      if (!_dataDirectory!.existsSync()) {
        _dataDirectory!.createSync(recursive: true);
      }
    } else {
      _dataDirectory = docDir;
    }
  }
  return _dataDirectory!;
}

Future<Directory> getAppDataDirectory() async {
  return getAppDirectory();
}
