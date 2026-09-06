#ifndef RUNNER_PROTOCOL_HOST_H_
#define RUNNER_PROTOCOL_HOST_H_

#include <string>

namespace flutter {
class FlutterEngine;
}

void RegisterProtocolChannel(flutter::FlutterEngine* engine);
void DestroyProtocolChannel();
void SetPendingProtocolUrl(const std::string& url);

#endif  // RUNNER_PROTOCOL_HOST_H_
