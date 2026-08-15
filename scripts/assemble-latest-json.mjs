#!/usr/bin/env node
/**
 * Assemble the Tauri updater manifest (latest.json) from release artifacts.
 *
 * Fail-closed by design: ANY missing platform payload or empty/missing .sig
 * aborts with a nonzero exit — a manifest that silently omits a platform
 * strands that platform's users on the old version (the updater only fetches
 * the key for its own platform and errors otherwise).
 *
 * Platform payloads (Tauri v2 bundle naming):
 *   darwin-aarch64  ← *.aarch64.app.tar.gz        (macOS arm64)
 *   darwin-x86_64   ← *.x64.app.tar.gz            (macOS Intel)
 *   windows-x86_64  ← *_x64-setup.exe             (NSIS per-user installer —
 *                       the updater payload; the MSI stays a download asset)
 */

import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const PLATFORM_MATCHERS = [
  // Tauri names payloads `${productName}_${version}_${arch}.app.tar.gz`
  // (underscore separators) and `${productName}_${version}_x64-setup.exe`.
  ["darwin-aarch64", (f) => /(?:^|[_-])aarch64\.app\.tar\.gz$/i.test(f)],
  ["darwin-x86_64", (f) => /(?:^|[_-])x64\.app\.tar\.gz$/i.test(f)],
  ["windows-x86_64", (f) => /(?:^|[_-])x64-setup\.exe$/i.test(f)],
];

/**
 * Pure assembler. `files` maps artifact filename → signature string or null
 * (no sibling .sig). Returns the manifest object; throws on any violation.
 */
export function assembleLatestJson({ files, tag, repo, pubDate, notes }) {
  const version = tag.replace(/^v/, "");
  if (!/^\d+\.\d+\.\d+(-[\w.]+)?$/.test(version)) {
    throw new Error(`tag ${tag} does not look like vX.Y.Z — refusing to publish`);
  }
  const baseUrl = `https://github.com/${repo}/releases/download/${encodeURIComponent(tag)}`;
  const platforms = {};
  for (const [platform, matches] of PLATFORM_MATCHERS) {
    const hits = [...files.keys()].filter(matches);
    if (hits.length === 0) {
      throw new Error(`no updater payload found for ${platform} — refusing to publish`);
    }
    if (hits.length > 1) {
      throw new Error(
        `ambiguous updater payload for ${platform}: ${hits.join(", ")} — refusing to publish`,
      );
    }
    const artifact = hits[0];
    const signature = files.get(artifact);
    if (typeof signature !== "string" || signature.trim().length === 0) {
      throw new Error(`missing or empty signature for ${artifact} (${platform}) — refusing to publish`);
    }
    platforms[platform] = {
      signature: signature.trim(),
      url: `${baseUrl}/${encodeURIComponent(artifact)}`,
    };
  }
  return {
    version,
    notes: notes ?? `Release ${tag}`,
    pub_date: pubDate,
    platforms,
  };
}

/** Read every file in `dir` into the filename → sig-content map the pure
 *  assembler consumes (.sig files attach to their base artifact). */
export function collectArtifacts(dir) {
  const names = readdirSync(dir);
  const sigs = new Map(
    names
      .filter((n) => n.endsWith(".sig"))
      .map((n) => [n.slice(0, -".sig".length), readFileSync(join(dir, n), "utf8")]),
  );
  const files = new Map();
  for (const n of names) {
    if (n.endsWith(".sig")) continue;
    files.set(n, sigs.has(n) ? sigs.get(n) : null);
  }
  return files;
}

async function main() {
  const [dir, tag, repo, outPath] = process.argv.slice(2);
  if (!dir || !tag || !repo || !outPath) {
    console.error("usage: assemble-latest-json.mjs <artifacts-dir> <tag> <owner/repo> <out-file>");
    process.exit(2);
  }
  const manifest = assembleLatestJson({
    files: collectArtifacts(dir),
    tag,
    repo,
    pubDate: new Date().toISOString().replace(/\.\d+Z$/, "Z"),
  });
  writeFileSync(outPath, JSON.stringify(manifest, null, 2) + "\n");
  console.log(`wrote ${outPath} (${Object.keys(manifest.platforms).join(", ")})`);
}

const invokedAsScript =
  process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href;
if (invokedAsScript) {
  await main();
}
