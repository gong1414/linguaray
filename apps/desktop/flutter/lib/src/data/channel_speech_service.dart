import 'dart:async';

import 'package:flutter/services.dart';
import 'package:linguaray_application/linguaray_application.dart';

final class ChannelSpeechService implements SpeechService {
  ChannelSpeechService({MethodChannel? channel})
    : _channel = channel ?? const MethodChannel('linguaray/speech') {
    _channel.setMethodCallHandler(_handleNativeCall);
  }

  final MethodChannel _channel;
  final StreamController<SpeechState> _states =
      StreamController<SpeechState>.broadcast(sync: true);

  @override
  Stream<SpeechState> get states => _states.stream;

  @override
  Future<bool> isAvailable() async {
    try {
      final available = await _channel.invokeMethod<bool>('isAvailable');
      return available ?? false;
    } catch (_) {
      return false;
    }
  }

  @override
  Future<SpeechState> speak({
    required String text,
    required SpeechUtteranceKind kind,
    String? language,
  }) async {
    final trimmed = text.trim();
    if (trimmed.isEmpty) return const SpeechState.idle();
    try {
      final available = await isAvailable();
      if (!available) {
        return _emit(
          SpeechState(
            status: SpeechStatus.unavailable,
            errorCode: AppErrorCode.speechUnavailable.wireName,
          ),
        );
      }
      await _channel.invokeMethod<void>('speak', {
        'text': trimmed,
        'kind': kind.name,
        if (language != null) 'language': language,
      });
      return _emit(const SpeechState(status: SpeechStatus.speaking));
    } on PlatformException catch (error) {
      if (error.code == 'interrupted') {
        return _emit(
          SpeechState(
            status: SpeechStatus.interrupted,
            errorCode: AppErrorCode.speechInterrupted.wireName,
          ),
        );
      }
      return _emit(
        SpeechState(
          status: SpeechStatus.failed,
          errorCode: AppErrorCode.speechFailed.wireName,
        ),
      );
    } catch (_) {
      return _emit(
        SpeechState(
          status: SpeechStatus.failed,
          errorCode: AppErrorCode.speechFailed.wireName,
        ),
      );
    }
  }

  @override
  Future<SpeechState> stop() async {
    try {
      await _channel.invokeMethod<void>('stop');
    } catch (_) {}
    return _emit(const SpeechState.idle());
  }

  @override
  Future<void> dispose() async {
    _channel.setMethodCallHandler(null);
    await _states.close();
  }

  SpeechState _emit(SpeechState state) {
    if (!_states.isClosed) _states.add(state);
    return state;
  }

  Future<dynamic> _handleNativeCall(MethodCall call) async {
    if (call.method != 'stateChanged') return;
    final value = call.arguments as String?;
    switch (value) {
      case 'idle':
        _emit(const SpeechState.idle());
      case 'interrupted':
        _emit(
          SpeechState(
            status: SpeechStatus.interrupted,
            errorCode: AppErrorCode.speechInterrupted.wireName,
          ),
        );
      case 'failed':
        _emit(
          SpeechState(
            status: SpeechStatus.failed,
            errorCode: AppErrorCode.speechFailed.wireName,
          ),
        );
      default:
        break;
    }
  }
}
