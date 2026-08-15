/** Typed wrappers for the offline-dictionary Rust commands. */
import { invoke } from "../../../bridge/invoke";
import type { DictPackage, DictResult } from "./copy";

export const dictLookup = (word: string): Promise<DictResult> =>
  invoke<DictResult>("dict_lookup", { word });

export const dictListPackages = (): Promise<DictPackage[]> =>
  invoke<DictPackage[]>("dict_list_packages");

export const dictInstallPackage = (
  sourceDir: string,
  packageId: string,
  name: string,
  version: string,
): Promise<void> =>
  invoke<void>("dict_install_package", { sourceDir, packageId, name, version });
