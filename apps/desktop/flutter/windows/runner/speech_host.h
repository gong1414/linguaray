#ifndef RUNNER_SPEECH_HOST_H_
#define RUNNER_SPEECH_HOST_H_

#include <windows.h>

namespace flutter {
class FlutterEngine;
}

void RegisterSpeechChannel(flutter::FlutterEngine* engine, HWND window);
void DestroySpeechChannel();
bool HandleSpeechEvent(HWND hwnd, UINT message);

#endif  // RUNNER_SPEECH_HOST_H_
