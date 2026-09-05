import 'dart:io';

import 'package:flutter/services.dart';

final class SelectionTarget {
  const SelectionTarget({required this.id, required this.text});
  final String id;
  final String text;
}

enum SelectionReplacementResult { replaced, changed, denied, unsupported }

class SelectionReplacementController {
  static const _channel = MethodChannel('linguaray/selection_replacement');

  Future<SelectionTarget?> capture() async {
    if (!Platform.isMacOS) return null;
    try {
      final value = await _channel.invokeMapMethod<String, String>('capture');
      if (value?['id'] == null || value?['text'] == null) return null;
      return SelectionTarget(id: value!['id']!, text: value['text']!);
    } on PlatformException {
      return null;
    } on MissingPluginException {
      return null;
    }
  }

  Future<SelectionReplacementResult> replace(
    SelectionTarget target,
    String text,
  ) async {
    if (!Platform.isMacOS) return SelectionReplacementResult.unsupported;
    try {
      final value = await _channel.invokeMethod<String>('replace', {
        'id': target.id,
        'text': text,
      });
      return switch (value) {
        'replaced' => SelectionReplacementResult.replaced,
        'permission_denied' => SelectionReplacementResult.denied,
        'selection_changed' => SelectionReplacementResult.changed,
        _ => SelectionReplacementResult.unsupported,
      };
    } on PlatformException {
      return SelectionReplacementResult.unsupported;
    }
  }
}

final selectionReplacementController = SelectionReplacementController();
