#include "speech_host.h"

#include <flutter/flutter_engine.h>
#include <flutter/method_channel.h>
#include <flutter/standard_method_codec.h>
#include <memory>
#include <sapi.h>
#include <sphelper.h>
#include <string>
#include <variant>

namespace {
constexpr char kSpeechChannel[] = "linguaray/speech";
constexpr UINT kSpeechEventMessage = WM_APP + 0x42;
ISpVoice* g_voice = nullptr;
ULONG g_active_speech_stream = 0;
HWND g_speech_window = nullptr;
flutter::MethodChannel<flutter::EncodableValue>* g_speech_channel = nullptr;

void EnsureVoice() {
  if (g_voice != nullptr) {
    return;
  }
  const HRESULT result = ::CoCreateInstance(
      CLSID_SpVoice, nullptr, CLSCTX_ALL, IID_ISpVoice,
      reinterpret_cast<void**>(&g_voice));
  if (FAILED(result)) {
    g_voice = nullptr;
    return;
  }
  if (g_speech_window != nullptr) {
    g_voice->SetNotifyWindowMessage(g_speech_window, kSpeechEventMessage, 0,
                                    0);
    const ULONGLONG interest = SPFEI(SPEI_END_INPUT_STREAM);
    g_voice->SetInterest(interest, interest);
  }
}
}  // namespace

void RegisterSpeechChannel(flutter::FlutterEngine* engine, HWND window) {
  g_speech_window = window;
  auto speech = std::make_unique<flutter::MethodChannel<flutter::EncodableValue>>(
      engine->messenger(), kSpeechChannel,
      &flutter::StandardMethodCodec::GetInstance());
  g_speech_channel = speech.get();
  speech->SetMethodCallHandler(
      [](const flutter::MethodCall<flutter::EncodableValue>& call,
         std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result) {
        if (call.method_name() == "isAvailable") {
          EnsureVoice();
          result->Success(flutter::EncodableValue(g_voice != nullptr));
          return;
        }
        if (call.method_name() == "stop") {
          if (g_voice) {
            g_active_speech_stream = 0;
            g_voice->Speak(L"", SPF_PURGEBEFORESPEAK, nullptr);
          }
          result->Success();
          return;
        }
        if (call.method_name() == "speak") {
          EnsureVoice();
          if (g_voice == nullptr) {
            result->Error("unavailable", "System speech is unavailable.");
            return;
          }
          const auto* args =
              std::get_if<flutter::EncodableMap>(call.arguments());
          std::string text;
          if (args != nullptr) {
            const auto it = args->find(flutter::EncodableValue("text"));
            if (it != args->end()) {
              if (const auto* value = std::get_if<std::string>(&it->second)) {
                text = *value;
              }
            }
          }
          if (text.empty()) {
            result->Error("bad_args", "Expected text.");
            return;
          }
          const int size =
              MultiByteToWideChar(CP_UTF8, 0, text.c_str(), -1, nullptr, 0);
          std::wstring wide(static_cast<size_t>(size), L'\0');
          MultiByteToWideChar(CP_UTF8, 0, text.c_str(), -1, wide.data(), size);
          ULONG stream_number = 0;
          const HRESULT spoken =
              g_voice->Speak(wide.c_str(), SPF_ASYNC | SPF_PURGEBEFORESPEAK,
                             &stream_number);
          if (FAILED(spoken)) {
            result->Error("failed", "System speech could not start.");
            return;
          }
          g_active_speech_stream = stream_number;
          result->Success();
          return;
        }
        result->NotImplemented();
      });
  speech.release();
}

void DestroySpeechChannel() {
  g_speech_channel = nullptr;
  g_speech_window = nullptr;
  if (g_voice != nullptr) {
    g_active_speech_stream = 0;
    g_voice->Release();
    g_voice = nullptr;
  }
}

bool HandleSpeechEvent(HWND hwnd, UINT message) {
  (void)hwnd;
  if (message != kSpeechEventMessage) {
    return false;
  }
  if (g_voice == nullptr) {
    return true;
  }
  SPEVENT event{};
  ULONG fetched = 0;
  while (g_voice->GetEvents(1, &event, &fetched) == S_OK && fetched > 0) {
    if (event.eEventId == SPEI_END_INPUT_STREAM &&
        event.ulStreamNum == g_active_speech_stream &&
        g_speech_channel != nullptr) {
      g_active_speech_stream = 0;
      g_speech_channel->InvokeMethod(
          "stateChanged",
          std::make_unique<flutter::EncodableValue>(std::string("idle")));
    }
    SpClearEvent(&event);
    fetched = 0;
  }
  return true;
}
