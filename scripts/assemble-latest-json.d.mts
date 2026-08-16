/** Type surface for scripts/assemble-latest-json.mjs (test-imported). */

export type UpdaterManifest = {
  version: string;
  notes: string;
  pub_date: string;
  platforms: Record<string, { signature: string; url: string }>;
};

export function assembleLatestJson(input: {
  /** artifact filename → signature content (null when no sibling .sig). */
  files: Map<string, string | null>;
  tag: string;
  repo: string;
  pubDate: string;
  notes?: string;
}): UpdaterManifest;

export function collectArtifacts(dir: string): Map<string, string | null>;
