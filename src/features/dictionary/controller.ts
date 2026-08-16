/** Dictionary controller — lookup + offline package install. */
import { useCallback, useEffect, useRef, useState } from "react";
import { detectLocale } from "../../app/i18n";
import { DICTIONARY_COPY, type DictPackage, type DictResult } from "./copy";
import * as ipc from "./ipc";

export function useDictionaryController() {
  const t = DICTIONARY_COPY[detectLocale()];
  const [word, setWord] = useState("");
  const [result, setResult] = useState<DictResult>(null);
  const [miss, setMiss] = useState(false);
  const [packages, setPackages] = useState<DictPackage[]>([]);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [sourceDir, setSourceDir] = useState("");
  const [packageId, setPackageId] = useState("");
  const [packageName, setPackageName] = useState("");
  const [version, setVersion] = useState("1.0");
  const [installing, setInstalling] = useState(false);
  const cancelledRef = useRef(false);

  const reloadPackages = useCallback(async () => {
    const listed = await ipc.dictListPackages();
    if (!cancelledRef.current) setPackages(listed);
  }, []);

  useEffect(() => {
    cancelledRef.current = false;
    void reloadPackages().catch((e) => !cancelledRef.current && setError(String(e)));
    return () => {
      cancelledRef.current = true;
    };
  }, [reloadPackages]);

  const lookup = useCallback(async () => {
    setMiss(false);
    setResult(null);
    setError("");
    try {
      const found = await ipc.dictLookup(word.trim());
      if (found) setResult(found);
      else setMiss(true);
    } catch (e) {
      setError(String(e));
    }
  }, [word]);

  const install = useCallback(async () => {
    setError("");
    setNotice("");
    setInstalling(true);
    try {
      await ipc.dictInstallPackage(
        sourceDir.trim(),
        packageId.trim(),
        packageName.trim() || packageId.trim(),
        version.trim() || "1.0",
      );
      setNotice(t.installed);
      await reloadPackages();
    } catch (e) {
      setError(String(e));
    } finally {
      if (!cancelledRef.current) setInstalling(false);
    }
  }, [sourceDir, packageId, packageName, version, reloadPackages, t.installed]);

  return {
    word,
    setWord,
    result,
    miss,
    packages,
    error,
    notice,
    sourceDir,
    setSourceDir,
    packageId,
    setPackageId,
    packageName,
    setPackageName,
    version,
    setVersion,
    installing,
    lookup: () => void lookup(),
    install: () => void install(),
  };
}

export type DictionaryController = ReturnType<typeof useDictionaryController>;
