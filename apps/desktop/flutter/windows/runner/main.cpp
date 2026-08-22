#include <flutter/dart_project.h>
#include <flutter/flutter_view_controller.h>
#include <flutter/generated_plugin_registrant.h>
#include <dwmapi.h>
#include <windows.h>

#include "utils.h"
#include "flutter_window.h"

namespace {

void DisableRoundedCorners(HWND window) {
  DWORD process_id = 0;
  ::GetWindowThreadProcessId(window, &process_id);
  if (process_id != ::GetCurrentProcessId() ||
      ::GetAncestor(window, GA_ROOT) != window) {
    return;
  }

  const DWM_WINDOW_CORNER_PREFERENCE preference = DWMWCP_DONOTROUND;
  ::DwmSetWindowAttribute(window, DWMWA_WINDOW_CORNER_PREFERENCE, &preference,
                          sizeof(preference));
}

void CALLBACK HandleWindowCreatedOrShown(HWINEVENTHOOK, DWORD, HWND window,
                                         LONG object_id, LONG child_id, DWORD,
                                         DWORD) {
  if (window != nullptr && object_id == OBJID_WINDOW &&
      child_id == CHILDID_SELF) {
    DisableRoundedCorners(window);
  }
}

}  // namespace

int APIENTRY wWinMain(_In_ HINSTANCE instance, _In_opt_ HINSTANCE prev,
                      _In_ wchar_t *command_line, _In_ int show_command) {
  // TODO: Re-implement protocol handler dispatch when feature is restored
  // Attach to console when present (e.g., 'flutter run') or create a
  // new console when running with a debugger.
  if (!::AttachConsole(ATTACH_PARENT_PROCESS) && ::IsDebuggerPresent()) {
    CreateAndAttachConsole();
  }

  // Initialize COM, so that it is available for use in the library and/or
  // plugins.
  ::CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED);

  // Flutter creates top-level windows after the engine starts. Apply the DWM
  // square-corner preference whenever this process's host window is created or
  // shown. The workbench and quick translator share that stable host window.
  const auto corner_hook = ::SetWinEventHook(
      EVENT_OBJECT_CREATE, EVENT_OBJECT_SHOW, nullptr,
      HandleWindowCreatedOrShown, ::GetCurrentProcessId(), 0,
      WINEVENT_OUTOFCONTEXT);

  flutter::DartProject project(L"data");

  // Flutter's OpenGLES SDF Impeller backend currently produces noticeably
  // softer CJK glyphs on Windows. Keep the Windows desktop app on Skia until
  // the Impeller text renderer reaches comparable small-text quality.
  project.set_impeller_switch(flutter::ImpellerSwitch::Disabled);

  auto command_line_arguments{GetCommandLineArguments()};

  project.set_dart_entrypoint_arguments(std::move(command_line_arguments));

  FlutterWindow window(project);
  Win32Window::Point origin(10, 10);
  Win32Window::Size size(840, 560);
  if (!window.Create(L"LinguaRay", origin, size)) {
    return EXIT_FAILURE;
  }
  window.SetQuitOnClose(false);

  ::MSG msg;
  while (::GetMessage(&msg, nullptr, 0, 0)) {
    ::TranslateMessage(&msg);
    ::DispatchMessage(&msg);
  }

  if (corner_hook != nullptr) {
    ::UnhookWinEvent(corner_hook);
  }
  ::CoUninitialize();
  return EXIT_SUCCESS;
}
