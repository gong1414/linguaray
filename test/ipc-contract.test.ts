import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

/**
 * IPC contract lock (docs/IPC-CONTRACT.md). Reconciles four sources of truth:
 * Rust registration (tauri-specta `collect_commands!`) × frontend command call sites ×
 * capability authorization × the documented RETAINED allow-list. Any command
 * addition/removal must keep all four in sync — this test fails otherwise.
 */

const ROOT = join(__dirname, "..");

/**
 * Registered-but-uncalled commands must be explicitly retained here with a
 * docs/IPC-CONTRACT.md entry. Empty means every registered command has a
 * frontend caller (the steady state — enforced since refactor P0.3 removed
 * the five dead commands the audit found).
 */
const RETAINED_NO_FRONTEND_CALLER = new Set<string>([]);

function registeredCommands(): Set<string> {
  const lib = readFileSync(join(ROOT, "src-tauri/src/lib.rs"), "utf8").replace(/\/\/[^\n]*/g, "");
  const start = lib.indexOf("collect_commands!");
  const block = lib.slice(start, lib.indexOf("])", start));
  const names = new Set(
    [...block.matchAll(/\b([a-z][a-z_0-9]{2,})\b/g)].map((m) => m[1]),
  );
  names.delete("collect_commands");
  return names;
}

function frontendInvokes(): Set<string> {
  const called = new Set<string>();
  const bindings = readFileSync(join(ROOT, "src/bridge/bindings.ts"), "utf8")
    .split("/* Types */", 1)[0];
  const generatedCommands = new Map(
    [...bindings.matchAll(/^\s*([a-z][A-Za-z0-9]*):[\s\S]*?__TAURI_INVOKE[\s\S]*?\(\s*["'`]([a-z_0-9]+)["'`]/gm)]
      .map((m) => [m[1], m[2]]),
  );
  const walk = (dir: string) => {
    for (const name of readdirSync(dir)) {
      const p = join(dir, name);
      if (statSync(p).isDirectory()) walk(p);
      else if (/\.(ts|tsx)$/.test(name)) {
        const text = readFileSync(p, "utf8");
        // Keep detecting any legacy/raw command seam alongside generated wrappers.
        for (const m of text.matchAll(/invoke\s*(?:<[\s\S]*?>)?\s*\(\s*["'`]([a-z_0-9]+)["'`]/g)) {
          called.add(m[1]);
        }
        for (const m of text.matchAll(/\bcommands\.([a-z][A-Za-z0-9]*)\s*\(/g)) {
          const command = generatedCommands.get(m[1]);
          if (command) called.add(command);
        }
      }
    }
  };
  walk(join(ROOT, "src"));
  return called;
}

function authorizedCommands(): Set<string> {
  const allow = new Set<string>();
  const capDir = join(ROOT, "src-tauri/capabilities");
  for (const name of readdirSync(capDir)) {
    if (!name.endsWith(".json")) continue;
    const cap = JSON.parse(readFileSync(join(capDir, name), "utf8"));
    for (const p of cap.permissions as (string | { identifier: string })[]) {
      const id = typeof p === "string" ? p : p.identifier;
      if (id.startsWith("allow-")) allow.add(id.slice(6).replace(/-/g, "_"));
    }
  }
  return allow;
}

describe("IPC contract (docs/IPC-CONTRACT.md)", () => {
  const registered = registeredCommands();
  const called = frontendInvokes();
  const authorized = authorizedCommands();

  it("frontend never invokes an unregistered command", () => {
    expect([...called].filter((c) => !registered.has(c))).toEqual([]);
  });

  it("frontend never invokes an unauthorized command (fail-closed)", () => {
    expect([...called].filter((c) => !authorized.has(c))).toEqual([]);
  });

  it("capabilities never authorize an unregistered command (no stale allow-*)", () => {
    expect([...authorized].filter((c) => !registered.has(c))).toEqual([]);
  });

  it("every registered command has a caller or an explicit RETAINED entry", () => {
    expect([...registered].filter((c) => !called.has(c) && !RETAINED_NO_FRONTEND_CALLER.has(c)))
      .toEqual([]);
  });

  it("RETAINED entries must be real registered commands (no zombie allow-list)", () => {
    expect([...RETAINED_NO_FRONTEND_CALLER].filter((c) => !registered.has(c))).toEqual([]);
  });
});
