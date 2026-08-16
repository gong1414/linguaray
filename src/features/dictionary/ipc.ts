/** Typed wrappers for the offline-dictionary Rust commands. */
import { commands } from "../../bridge/invoke";
import type { DictPackage, DictResult } from "./copy";

export const dictLookup = (word: string): Promise<DictResult> =>
  commands.dictLookup(word);

export const dictListPackages = (): Promise<DictPackage[]> =>
  commands.dictListPackages();

export const dictInstallPackage = (
  sourceDir: string,
  packageId: string,
  name: string,
  version: string,
): Promise<void> =>
  commands.dictInstallPackage(sourceDir, packageId, name, version).then(() => undefined);
