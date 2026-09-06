import { Clipboard } from "@raycast/api";

import { openLinguaRay } from "./open-linguaray";

export default async function command(): Promise<void> {
  const text = await Clipboard.readText();
  if (text?.trim()) {
    await openLinguaRay("translate", text);
  } else {
    await openLinguaRay("clipboard-translate");
  }
}
