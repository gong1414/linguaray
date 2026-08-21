import { App as AntApp, theme } from "antd";
import { XProvider } from "@ant-design/x";
import enUS from "antd/locale/en_US";
import zhCN from "antd/locale/zh_CN";
import enUSX from "@ant-design/x/locale/en_US";
import zhCNX from "@ant-design/x/locale/zh_CN";
import { createContext, useContext, useEffect, useState, type ReactNode } from "react";
import "antd/dist/reset.css";
import "@ant-design/x-markdown/themes/light.css";
import "@ant-design/x-markdown/themes/dark.css";
import "../ui/styles.css";

type ColorScheme = "light" | "dark";

const AppColorSchemeContext = createContext<ColorScheme>("light");

export function useAppColorScheme(): ColorScheme {
  return useContext(AppColorSchemeContext);
}

function readColorScheme(): ColorScheme {
  const saved = localStorage.getItem("linguaray.theme");
  if (saved === "light" || saved === "dark") return saved;
  return window.matchMedia?.("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function isChineseLocale() {
  return (navigator.language || "en").toLowerCase().startsWith("zh");
}

/** Shared Ant Design X provider for every Tauri webview window. */
export function AppProviders({ children, transparent = false, forceColorScheme }: {
  children: ReactNode;
  transparent?: boolean;
  forceColorScheme?: ColorScheme;
}) {
  const [colorScheme, setColorScheme] = useState<ColorScheme>(readColorScheme);
  const activeColorScheme = forceColorScheme ?? colorScheme;

  useEffect(() => {
    const media = window.matchMedia?.("(prefers-color-scheme: dark)");
    if (!media) return;
    const onChange = (event: MediaQueryListEvent) => {
      const saved = localStorage.getItem("linguaray.theme");
      if (saved !== "light" && saved !== "dark") setColorScheme(event.matches ? "dark" : "light");
    };
    media.addEventListener("change", onChange);
    return () => media.removeEventListener("change", onChange);
  }, []);

  const locale = isChineseLocale() ? { ...zhCNX, ...zhCN } : { ...enUSX, ...enUS };

  return (
    <XProvider
      locale={locale}
      button={{ autoInsertSpace: false }}
      theme={{
        algorithm: activeColorScheme === "dark" ? theme.darkAlgorithm : theme.defaultAlgorithm,
        cssVar: {},
        token: {
          colorPrimary: "#0958d9",
          colorError: "#a8071a",
          colorInfo: activeColorScheme === "dark" ? "#69b1ff" : "#0958d9",
          colorInfoBg: activeColorScheme === "dark" ? "#111a2c" : "#e6f4ff",
          colorSuccess: activeColorScheme === "dark" ? "#73d13d" : "#237804",
          colorSuccessBg: activeColorScheme === "dark" ? "#162312" : "#f6ffed",
          colorWarning: activeColorScheme === "dark" ? "#ffd666" : "#874d00",
          colorWarningBg: activeColorScheme === "dark" ? "#2b2111" : "#fffbe6",
          colorLink: activeColorScheme === "dark" ? "#69b1ff" : "#0958d9",
          colorTextSecondary: activeColorScheme === "dark" ? "#bfbfbf" : "#595959",
          colorTextDescription: activeColorScheme === "dark" ? "#bfbfbf" : "#595959",
          colorWarningText: activeColorScheme === "dark" ? "#ffd666" : "#874d00",
          colorSuccessText: activeColorScheme === "dark" ? "#b7eb8f" : "#135200",
          borderRadius: 10,
          fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
        },
        components: {
          Menu: {
            itemSelectedColor: activeColorScheme === "dark" ? "#69b1ff" : "#0958d9",
          },
        },
      }}
    >
      <AppColorSchemeContext.Provider value={activeColorScheme}>
        <AntApp className="lr-ant-app">
          <div className={transparent ? "lr-root lr-root-transparent" : "lr-root"} data-color-scheme={activeColorScheme}>
            {children}
          </div>
        </AntApp>
      </AppColorSchemeContext.Provider>
    </XProvider>
  );
}
