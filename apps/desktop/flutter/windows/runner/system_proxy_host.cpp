#include "system_proxy_host.h"

#include <flutter/flutter_engine.h>
#include <flutter/method_channel.h>
#include <flutter/standard_method_codec.h>
#include <memory>
#include <string>
#include <vector>
#include <windows.h>
#include <wininet.h>

#include "utils.h"

namespace {
constexpr char kSystemProxyChannel[] = "linguaray/system_proxy";

std::string ProxyForScheme(const std::string& raw, const std::string& scheme) {
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
  HINTERNET internet = InternetOpenW(L"LinguaRay", INTERNET_OPEN_TYPE_PRECONFIG,
                                     nullptr, nullptr, 0);
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
      value[flutter::EncodableValue("https")] = flutter::EncodableValue(https);
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
}  // namespace

void RegisterSystemProxyChannel(flutter::FlutterEngine* engine) {
  auto system_proxy =
      std::make_unique<flutter::MethodChannel<flutter::EncodableValue>>(
          engine->messenger(), kSystemProxyChannel,
          &flutter::StandardMethodCodec::GetInstance());
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
  system_proxy.release();
}
