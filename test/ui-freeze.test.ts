import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

/**
 * UI freeze (docs/UI-RULES.md). Fluent UI is the only production component
 * framework; Ueli supplies the MIT-licensed renderer structure. These gates
 * keep the capability/view boundary and prevent a second UI kit from returning.
 */

const ROOT = join(__dirname, "..");

describe("UI freeze (Phase 5)", () => {
  it("the self-built @linguaray/ui kit stays deleted", () => {
    expect(
      existsSync(join(ROOT, "packages", "ui")),
      "packages/ui was deleted in migration Phase 5. Do not resurrect a parallel " +
        "component kit — compose Fluent UI controls (docs/UI-RULES.md).",
    ).toBe(false);
  });

  it("package.json uses Fluent UI and bans legacy/parallel UI kits", () => {
    const pkg = JSON.parse(readFileSync(join(ROOT, "package.json"), "utf8"));
    const deps = { ...pkg.dependencies, ...pkg.devDependencies };
    expect(deps["@fluentui/react-components"]).toBeDefined();
    expect(deps["@fluentui/react-icons"]).toBeDefined();
    for (const banned of [
      "solid-js",
      "lucide-solid",
      "lucide-react",
      "@linguaray/ui",
      "@mantine/core",
      "vite-plugin-solid",
      "postcss-preset-mantine",
    ]) {
      expect(deps[banned], `${banned} must stay removed`).toBeUndefined();
    }
  });

  it("production source has no legacy UI imports or native control substitutes", () => {
    const offenders: string[] = [];
    const walk = (dir: string) => {
      for (const name of readdirSync(dir)) {
        const p = join(dir, name);
        if (statSync(p).isDirectory()) walk(p);
        else if (/\.(tsx|ts)$/.test(name) && !/\.(test|stories)\./.test(name)) {
          const source = readFileSync(p, "utf8");
          if (/@mantine\/|lucide-react|<button\b|<select\b|<textarea\b/.test(source)) {
            offenders.push(p.slice(ROOT.length + 1));
          }
        }
      }
    };
    walk(join(ROOT, "src"));
    expect(offenders, "Use Fluent UI controls; do not revive legacy or native substitutes.").toEqual([]);
  });

  it("view modules do not import bridge or feature IPC implementations", () => {
    const offenders: string[] = [];
    const walk = (dir: string) => {
      for (const name of readdirSync(dir)) {
        const p = join(dir, name);
        if (statSync(p).isDirectory()) walk(p);
        else if (/\.tsx$/.test(name) && !/\.(test|stories)\./.test(name) && /(view|View|ProviderList|ProviderDetail)\.tsx$/.test(name)) {
          const source = readFileSync(p, "utf8");
          if (/from\s+["'][^"']*(?:bridge\/|(?:^|\/)[^/"']*ipc)["']/.test(source)) {
            offenders.push(p.slice(ROOT.length + 1));
          }
        }
      }
    };
    walk(join(ROOT, "src", "features"));
    expect(offenders, "Views receive callbacks; bridge/IPC belongs in controllers.").toEqual([]);
  });

  it("keeps the Ueli MIT source attribution", () => {
    const notices = readFileSync(join(ROOT, "THIRD_PARTY_NOTICES.md"), "utf8");
    expect(notices).toContain("https://github.com/oliverschwendener/ueli");
    expect(notices).toContain("f04ebdd82df71949d6b685ca7f2e5dd7e9b1bf90");
    expect(notices).toContain("Copyright (c) 2023 Oliver Schwendener");
  });

  it("keeps the adapted Ueli renderer layer and its per-file attribution", () => {
    const upstreamCommit = "f04ebdd82df71949d6b685ca7f2e5dd7e9b1bf90";
    for (const name of ["BaseLayout.tsx", "Header.tsx", "Footer.tsx", "Settings.tsx", "SearchResult.tsx"]) {
      const path = join(ROOT, "src", "ui", "ueli", name);
      expect(existsSync(path), `${name} is part of the fixed Ueli renderer adapter`).toBe(true);
      const source = readFileSync(path, "utf8");
      expect(source).toContain("Ueli");
      expect(source).toContain(upstreamCommit);
      expect(source).toContain("MIT");
    }
  });

  it("routes the main surfaces through the Ueli renderer adapters", () => {
    for (const relative of [
      "src/features/shell/view.tsx",
      "src/features/translation/InputPanelView.tsx",
      "src/features/translation/PopupView.tsx",
      "src/features/provider/parts/ProviderList.tsx",
      "src/features/provider/parts/ProviderDetail.tsx",
    ]) {
      const source = readFileSync(join(ROOT, relative), "utf8");
      expect(source, `${relative} must not fall back to a separately designed shell`).toMatch(/ui\/ueli/);
    }
  });

  it("production src/ has no WindowChrome / custom window chrome imports", () => {
    const offenders: string[] = [];
    const walk = (dir: string) => {
      for (const name of readdirSync(dir)) {
        const p = join(dir, name);
        if (statSync(p).isDirectory()) walk(p);
        else if (/\.(tsx|ts)$/.test(name)) {
          const text = readFileSync(p, "utf8");
          if (/WindowChrome/.test(text)) offenders.push(p.slice(ROOT.length + 1));
        }
      }
    };
    walk(join(ROOT, "src"));
    expect(offenders, "No custom window chrome in production code.").toEqual([]);
  });
});
