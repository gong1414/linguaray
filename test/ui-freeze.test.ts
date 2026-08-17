import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

/**
 * UI architecture gates (docs/UI-RULES.md). Ant Design X is the product UI
 * framework and Ant Design supplies its general-purpose controls. These tests
 * preserve the capability/view boundary and prevent another UI kit or a
 * hand-built component library from returning.
 */

const ROOT = join(__dirname, "..");

function productionSources() {
  const files: string[] = [];
  const walk = (dir: string) => {
    for (const name of readdirSync(dir)) {
      const path = join(dir, name);
      if (statSync(path).isDirectory()) walk(path);
      else if (/\.(tsx|ts)$/.test(name) && !/\.(test|stories)\./.test(name)) files.push(path);
    }
  };
  walk(join(ROOT, "src"));
  return files;
}

describe("Ant Design X UI architecture", () => {
  it("keeps the deleted in-house component kit deleted", () => {
    expect(
      existsSync(join(ROOT, "packages", "ui")),
      "Compose Ant Design X / Ant Design controls; do not recreate packages/ui.",
    ).toBe(false);
  });

  it("uses the Ant Design stack and bans parallel UI kits", () => {
    const pkg = JSON.parse(readFileSync(join(ROOT, "package.json"), "utf8"));
    const deps = { ...pkg.dependencies, ...pkg.devDependencies };
    expect(deps["@ant-design/x"]).toBeDefined();
    expect(deps.antd).toBeDefined();
    expect(deps["@ant-design/icons"]).toBeDefined();
    for (const banned of [
      "@fluentui/react-components",
      "@fluentui/react-icons",
      "@mantine/core",
      "@linguaray/ui",
      "lucide-react",
      "lucide-solid",
      "solid-js",
      "vite-plugin-solid",
      "postcss-preset-mantine",
    ]) {
      expect(deps[banned], `${banned} must stay removed`).toBeUndefined();
    }
  });

  it("production source has no legacy UI imports or visible native substitutes", () => {
    const offenders = productionSources()
      .filter((path) => {
        const source = readFileSync(path, "utf8");
        return /@fluentui\/|@mantine\/|lucide-react|ui\/ueli|<button\b|<select\b|<textarea\b/.test(source);
      })
      .map((path) => path.slice(ROOT.length + 1));
    expect(offenders, "Use Ant Design X / Ant Design controls for visible UI.").toEqual([]);
  });

  it("view modules do not import bridge or feature IPC implementations", () => {
    const offenders = productionSources()
      .filter((path) => /(view|View|ProviderList|ProviderDetail)\.tsx$/.test(path))
      .filter((path) => /from\s+["'][^"']*(?:bridge\/|(?:^|\/)[^/"']*ipc)["']/.test(readFileSync(path, "utf8")))
      .map((path) => path.slice(ROOT.length + 1));
    expect(offenders, "Views receive state and callbacks; bridge/IPC belongs in controllers.").toEqual([]);
  });

  it("uses Ant Design X conversation primitives on both translation surfaces", () => {
    const input = readFileSync(join(ROOT, "src/features/translation/InputPanelView.tsx"), "utf8");
    const popup = readFileSync(join(ROOT, "src/features/translation/PopupView.tsx"), "utf8");
    expect(input).toMatch(/from "@ant-design\/x"/);
    expect(input).toMatch(/<Sender\b/);
    expect(input).toMatch(/<Bubble\.List\b/);
    expect(popup).toMatch(/from "@ant-design\/x"/);
    expect(popup).toMatch(/<Bubble\.List\b/);
  });

  it("routes settings through the shared Ant Design layer", () => {
    expect(existsSync(join(ROOT, "src/ui/x/Settings.tsx"))).toBe(true);
    expect(existsSync(join(ROOT, "src/ui/ueli"))).toBe(false);
    for (const relative of [
      "src/features/shell/view.tsx",
      "src/features/provider/parts/ProviderDetail.tsx",
      "src/features/privacy/view.tsx",
      "src/features/shortcuts/view.tsx",
    ]) {
      expect(readFileSync(join(ROOT, relative), "utf8"), relative).toMatch(/ui\/x/);
    }
  });

  it("keeps the Ant Design X MIT source attribution", () => {
    const notices = readFileSync(join(ROOT, "THIRD_PARTY_NOTICES.md"), "utf8");
    expect(notices).toContain("https://github.com/ant-design/x");
    expect(notices).toContain("25aad7b9c13abeb165466d53b375d0f2ffe81fa0");
    expect(notices).toContain("Copyright (c) 2015-present Ant UED");
  });

  it("production src/ has no custom WindowChrome", () => {
    const offenders = productionSources()
      .filter((path) => /WindowChrome/.test(readFileSync(path, "utf8")))
      .map((path) => path.slice(ROOT.length + 1));
    expect(offenders, "Use native window chrome for normal windows.").toEqual([]);
  });
});
