import { createSignal, Match, Switch, type Component } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Button } from "@linguaray/ui";
import { detectLocale } from "./i18n";

type Step = "welcome" | "accessibility" | "provider" | "history" | "shortcuts" | "done";

const COPY = {
  en: {
    welcome: "Welcome to LinguaRay",
    welcomeBody: "A privacy-first translation tool for your menu bar.",
    start: "Get started",
    a11y: "Grant Accessibility so selection translate can read highlighted text.",
    openSettings: "Open System Settings",
    skip: "Skip",
    continue: "Continue",
    provider: "Add your first provider in Settings, then come back here.",
    history: "Enable encrypted translation history?",
    enable: "Enable",
    shortcuts: "Review default shortcuts in Settings, or keep the defaults.",
    done: "You're all set!",
    openApp: "Open settings",
    tray: "Minimize to tray",
  },
  zh: {
    welcome: "欢迎使用 LinguaRay",
    welcomeBody: "一款隐私优先的菜单栏翻译工具。",
    start: "开始使用",
    a11y: "请授予辅助功能权限，以便划词翻译读取选中文本。",
    openSettings: "打开系统设置",
    skip: "跳过",
    continue: "继续",
    provider: "请先在设置中添加服务商，然后回到这里。",
    history: "是否启用加密翻译历史？",
    enable: "启用",
    shortcuts: "可在设置中查看默认快捷键，或继续使用默认值。",
    done: "设置完成！",
    openApp: "打开设置",
    tray: "最小化到托盘",
  },
};

const Onboarding: Component = () => {
  const t = COPY[detectLocale()];
  const [step, setStep] = createSignal<Step>("welcome");

  const advance = async (event: string) => {
    const next = await invoke<Step>("onboarding_next", { step: step(), event });
    setStep(next);
    if (next === "done") {
      await invoke("onboarding_complete");
    }
  };

  const finish = async (openSettings: boolean) => {
    await invoke("onboarding_complete");
    const win = getCurrentWindow();
    if (openSettings) {
      await invoke("open_settings_window", { section: "provider-center" });
    }
    await win.hide();
  };

  return (
    <main style={{ padding: "24px", "font-family": "system-ui" }}>
      <Switch>
        <Match when={step() === "welcome"}>
          <h1>{t.welcome}</h1>
          <p>{t.welcomeBody}</p>
          <Button onClick={() => void advance("start")}>{t.start}</Button>
        </Match>
        <Match when={step() === "accessibility"}>
          <p>{t.a11y}</p>
          <Button
            onClick={() =>
              void import("@tauri-apps/plugin-opener")
                .then(({ openUrl }) =>
                  openUrl(
                    "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
                  ),
                )
                .catch(() => {})
            }
          >
            {t.openSettings}
          </Button>
          <Button variant="ghost" onClick={() => void advance("skip")}>{t.skip}</Button>
          <Button onClick={() => void advance("continue")}>{t.continue}</Button>
        </Match>
        <Match when={step() === "provider"}>
          <p>{t.provider}</p>
          <Button onClick={() => void advance("continue")}>{t.continue}</Button>
          <Button variant="ghost" onClick={() => void advance("skip")}>{t.skip}</Button>
        </Match>
        <Match when={step() === "history"}>
          <p>{t.history}</p>
          <Button
            onClick={() => {
              void invoke("history_set_enabled", { enabled: true });
              void advance("continue");
            }}
          >
            {t.enable}
          </Button>
          <Button variant="ghost" onClick={() => void advance("skip")}>{t.skip}</Button>
        </Match>
        <Match when={step() === "shortcuts"}>
          <p>{t.shortcuts}</p>
          <Button onClick={() => void advance("complete")}>{t.continue}</Button>
        </Match>
        <Match when={step() === "done"}>
          <h1>{t.done}</h1>
          <Button onClick={() => void finish(true)}>{t.openApp}</Button>
          <Button variant="ghost" onClick={() => void finish(false)}>{t.tray}</Button>
        </Match>
      </Switch>
    </main>
  );
};

export default Onboarding;
