import 'dart:async';

import 'package:linguaray_runtime/linguaray_runtime.dart';

import 'runtime.dart';

/// Reads native translation events through asynchronous UniFFI methods.
///
/// Usage:
/// ```dart
/// final stream = LlmStream.translate(
///   providerId: 'openai',
///   sourceLang: 'en',
///   targetLang: 'zh',
///   text: 'Hello world',
/// );
/// await for (final chunk in stream) {
///   print(chunk.content);
/// }
/// ```
class LlmStream {
  /// Creates a Dart stream from an LLM streaming translation call.
  ///
  /// The stream yields [StreamChunkData] items with incremental text content.
  /// Completes normally when the LLM finishes, or with an error on failure.
  static Stream<StreamChunkData> translate({
    required String providerId,
    required String sourceLang,
    required String targetLang,
    required String text,
  }) {
    TranslationTask? task;
    var cancelled = false;
    final controller = StreamController<StreamChunkData>(
      onCancel: () {
        cancelled = true;
        task?.cancel();
      },
    );

    Future<void> read() async {
      try {
        final active = runtime
            .llm(providerId: providerId)
            .startTranslation(
              sourceLang: sourceLang,
              targetLang: targetLang,
              text: text,
            );
        task = active;
        if (cancelled) {
          active.cancel();
          return;
        }
        while (!cancelled) {
          final event = await active.next();
          if (cancelled) return;
          if (event == null) {
            throw const FormatException('stream ended before completion');
          }
          if (event.error != null) throw Exception(event.error);
          controller.add(
            StreamChunkData(
              content: event.content,
              isDone: event.finishReason != null,
              finishReason: event.finishReason,
            ),
          );
          if (event.finishReason != null) break;
        }
      } catch (error, stack) {
        if (!cancelled) controller.addError(error, stack);
      } finally {
        if (!controller.isClosed) unawaited(controller.close());
      }
    }

    controller.onListen = () => unawaited(read());

    return controller.stream;
  }
}

/// A single chunk of streaming translation data.
class StreamChunkData {
  final String content;
  final bool isDone;
  final String? finishReason;

  const StreamChunkData({
    required this.content,
    required this.isDone,
    this.finishReason,
  });
}
