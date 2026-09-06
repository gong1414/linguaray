#ifndef RUNNER_SYSTEM_PROXY_HOST_H_
#define RUNNER_SYSTEM_PROXY_HOST_H_

namespace flutter {
class FlutterEngine;
}

void RegisterSystemProxyChannel(flutter::FlutterEngine* engine);

#endif  // RUNNER_SYSTEM_PROXY_HOST_H_
