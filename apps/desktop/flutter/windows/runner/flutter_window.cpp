#include "flutter_window.h"

#include <optional>
#include <string>

#include "flutter/generated_plugin_registrant.h"
#include "protocol_host.h"
#include "speech_host.h"
#include "system_proxy_host.h"

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
  RegisterSpeechChannel(flutter_controller_->engine(), GetHandle());
  RegisterProtocolChannel(flutter_controller_->engine());
  RegisterSystemProxyChannel(flutter_controller_->engine());
  SetChildContent(flutter_controller_->view()->GetNativeWindow());

  // LinguaRay is tray-resident. Keep the stable host hidden at launch; Dart
  // presents it only for Settings or the transient translator. A redraw is
  // still required so the hidden Flutter surface is ready for the first tray
  // action without a blank frame.
  flutter_controller_->ForceRedraw();

  return true;
}

void FlutterWindow::OnDestroy() {
  DestroySpeechChannel();
  DestroyProtocolChannel();
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
    default:
      if (HandleSpeechEvent(hwnd, message)) {
        return 0;
      }
      break;
  }

  return Win32Window::MessageHandler(hwnd, message, wparam, lparam);
}
