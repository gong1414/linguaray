#include "protocol_host.h"

#include <flutter/flutter_engine.h>
#include <flutter/method_channel.h>
#include <flutter/standard_method_codec.h>
#include <memory>
#include <string>

namespace {
constexpr char kProtocolChannel[] = "linguaray/protocol";
flutter::MethodChannel<flutter::EncodableValue>* g_protocol_channel = nullptr;
std::string g_pending_protocol;
}  // namespace

void RegisterProtocolChannel(flutter::FlutterEngine* engine) {
  auto protocol =
      std::make_unique<flutter::MethodChannel<flutter::EncodableValue>>(
          engine->messenger(), kProtocolChannel,
          &flutter::StandardMethodCodec::GetInstance());
  g_protocol_channel = protocol.get();
  if (!g_pending_protocol.empty()) {
    g_protocol_channel->InvokeMethod(
        "open", std::make_unique<flutter::EncodableValue>(g_pending_protocol));
    g_pending_protocol.clear();
  }
  protocol.release();
}

void DestroyProtocolChannel() { g_protocol_channel = nullptr; }

void SetPendingProtocolUrl(const std::string& url) {
  if (g_protocol_channel != nullptr) {
    g_protocol_channel->InvokeMethod(
        "open", std::make_unique<flutter::EncodableValue>(url));
    return;
  }
  g_pending_protocol = url;
}
