#include "flutter_window.h"

#include <flutter/method_channel.h>
#include <flutter/standard_method_codec.h>
#include <optional>
#include <sapi.h>
#include <sphelper.h>
#include <string>
#include <variant>
#include <vector>
#include <windows.h>
#include <wininet.h>

#include "flutter/generated_plugin_registrant.h"
#include "utils.h"

namespace {
constexpr char kSpeechChannel[] = "linguaray/speech";
constexpr char kProtocolChannel[] = "linguaray/protocol";
constexpr char kSystemProxyChannel[] = "linguaray/system_proxy";
constexpr UINT kSpeechEventMessage = WM_APP + 0x42;
ISpVoice* g_voice = nullptr;
ULONG g_active_speech_stream = 0;
HWND g_speech_window = nullptr;
flutter::MethodChannel<flutter::EncodableValue>* g_speech_channel = nullptr;
flutter::MethodChannel<flutter::EncodableValue>* g_protocol_channel = nullptr;
std::string g_pending_protocol;

std::string ProxyForScheme(const std::string& raw,
                           const std::string& scheme) {
  if (raw.find('=') == std::string::npos) {
    return raw;
  }
  size_t offset = 0;
  while (offset < raw.size()) {
    const size_t separator = raw.find(';', offset);
    const std::string entry = raw.substr(
        offset, separator == std::string::npos ? std::string::npos
                                                : separator - offset);
    const size_t equals = entry.find('=');
    if (equals != std::string::npos && entry.substr(0, equals) == scheme) {
      return entry.substr(equals + 1);
    }
    if (separator == std::string::npos) {
      break;
    }
    offset = separator + 1;
  }
  return {};
}

flutter::EncodableMap ReadSystemProxy() {
  flutter::EncodableMap value;
  HINTERNET internet = InternetOpenW(L"LinguaRay",
                                     INTERNET_OPEN_TYPE_PRECONFIG, nullptr,
                                     nullptr, 0);
  if (internet == nullptr) {
    return value;
  }
  DWORD size = 0;
  InternetQueryOptionW(internet, INTERNET_OPTION_PROXY, nullptr, &size);
  std::vector<unsigned char> buffer(size);
  if (size == 0 ||
      !InternetQueryOptionW(internet, INTERNET_OPTION_PROXY, buffer.data(),
                            &size)) {
    InternetCloseHandle(internet);
    return value;
  }
  const auto* proxy =
      reinterpret_cast<const INTERNET_PROXY_INFO*>(buffer.data());
  if (proxy->dwAccessType == INTERNET_OPEN_TYPE_PROXY &&
      proxy->lpszProxy != nullptr) {
    const std::string raw = Utf8FromUtf16(proxy->lpszProxy);
    const std::string http = ProxyForScheme(raw, "http");
    const std::string https = ProxyForScheme(raw, "https");
    if (!http.empty()) {
      value[flutter::EncodableValue("http")] = flutter::EncodableValue(http);
    }
    if (!https.empty()) {
      value[flutter::EncodableValue("https")] =
          flutter::EncodableValue(https);
    }
  }
  flutter::EncodableList bypass;
  if (proxy->lpszProxyBypass != nullptr) {
    const std::string raw = Utf8FromUtf16(proxy->lpszProxyBypass);
    size_t offset = 0;
    while (offset < raw.size()) {
      const size_t separator = raw.find(';', offset);
      const std::string entry = raw.substr(
          offset, separator == std::string::npos ? std::string::npos
                                                  : separator - offset);
      if (!entry.empty()) {
        bypass.emplace_back(entry);
      }
      if (separator == std::string::npos) {
        break;
      }
      offset = separator + 1;
    }
  }
  value[flutter::EncodableValue("bypass")] = flutter::EncodableValue(bypass);
  InternetCloseHandle(internet);
  return value;
}

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

void RegisterHostChannels(flutter::FlutterEngine* engine, HWND window) {
  g_speech_window = window;
  const auto messenger = engine->messenger();
  const auto& codec = flutter::StandardMethodCodec::GetInstance();

  auto speech = std::make_unique<flutter::MethodChannel<flutter::EncodableValue>>(
      messenger, kSpeechChannel, &codec);
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
          const auto* args = std::get_if<flutter::EncodableMap>(call.arguments());
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
          const int size = MultiByteToWideChar(CP_UTF8, 0, text.c_str(), -1, nullptr, 0);
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

  auto protocol =
      std::make_unique<flutter::MethodChannel<flutter::EncodableValue>>(
          messenger, kProtocolChannel, &codec);
  auto system_proxy =
      std::make_unique<flutter::MethodChannel<flutter::EncodableValue>>(
          messenger, kSystemProxyChannel, &codec);
  system_proxy->SetMethodCallHandler(
      [](const flutter::MethodCall<flutter::EncodableValue>& call,
         std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>>
             result) {
        if (call.method_name() == "read") {
          result->Success(flutter::EncodableValue(ReadSystemProxy()));
          return;
        }
        result->NotImplemented();
      });
  g_protocol_channel = protocol.get();
  if (!g_pending_protocol.empty()) {
    g_protocol_channel->InvokeMethod(
        "open", std::make_unique<flutter::EncodableValue>(g_pending_protocol));
    g_pending_protocol.clear();
  }
  protocol.release();
  system_proxy.release();
  speech.release();
}
}  // namespace

void SetPendingProtocolUrl(const std::string& url) {
  if (g_protocol_channel != nullptr) {
    g_protocol_channel->InvokeMethod(
        "open", std::make_unique<flutter::EncodableValue>(url));
    return;
  }
  g_pending_protocol = url;
}

FlutterWindow::FlutterWindow(const flutter::DartProject& project)
    : project_(project) {}

FlutterWindow::~FlutterWindow() {}

bool FlutterWindow::OnCreate() {
  if (!Win32Window::OnCreate()) {
    return false;
  }

  RECT frame = GetClientArea();

  // The size here must match the window dimensions to avoid unnecessary surface
  // creation / destruction in the startup path.
  flutter_controller_ = std::make_unique<flutter::FlutterViewController>(
      frame.right - frame.left, frame.bottom - frame.top, project_);
  // Ensure that basic setup of the controller was successful.
  if (!flutter_controller_->engine() || !flutter_controller_->view()) {
    return false;
  }
  RegisterPlugins(flutter_controller_->engine());
  RegisterHostChannels(flutter_controller_->engine(), GetHandle());
  SetChildContent(flutter_controller_->view()->GetNativeWindow());

  // LinguaRay is tray-resident. Keep the stable host hidden at launch; Dart
  // presents it only for Settings or the transient translator. A redraw is
  // still required so the hidden Flutter surface is ready for the first tray
  // action without a blank frame.
  flutter_controller_->ForceRedraw();

  return true;
}

void FlutterWindow::OnDestroy() {
  g_speech_channel = nullptr;
  g_protocol_channel = nullptr;
  g_speech_window = nullptr;
  if (g_voice != nullptr) {
    g_active_speech_stream = 0;
    g_voice->Release();
    g_voice = nullptr;
  }
  if (flutter_controller_) {
    flutter_controller_ = nullptr;
  }

  Win32Window::OnDestroy();
}

LRESULT
FlutterWindow::MessageHandler(HWND hwnd, UINT const message,
                              WPARAM const wparam,
                              LPARAM const lparam) noexcept {
  // Give Flutter, including plugins, an opportunity to handle window messages.
  if (flutter_controller_) {
    std::optional<LRESULT> result =
        flutter_controller_->HandleTopLevelWindowProc(hwnd, message, wparam,
                                                      lparam);
    if (result) {
      return *result;
    }
  }

  switch (message) {
    case kSpeechEventMessage: {
      if (g_voice == nullptr) {
        return 0;
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
              std::make_unique<flutter::EncodableValue>(
                  std::string("idle")));
        }
        SpClearEvent(&event);
        fetched = 0;
      }
      return 0;
    }
    case WM_CLOSE:
      ::ShowWindow(hwnd, SW_HIDE);
      return 0;
    case WM_COPYDATA: {
      const auto* data = reinterpret_cast<COPYDATASTRUCT*>(lparam);
      if (data != nullptr && data->lpData != nullptr && data->cbData > 0) {
        const auto* bytes = static_cast<const char*>(data->lpData);
        size_t length = data->cbData;
        if (bytes[length - 1] == '\0') {
          --length;
        }
        SetPendingProtocolUrl(std::string(bytes, length));
      }
      return 1;
    }
    case WM_FONTCHANGE:
      flutter_controller_->engine()->ReloadSystemFonts();
      break;
  }

  return Win32Window::MessageHandler(hwnd, message, wparam, lparam);
}
