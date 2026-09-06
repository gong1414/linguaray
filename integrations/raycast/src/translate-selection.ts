import { getSelectedText } from "@raycast/api";

import { openLinguaRay } from "./open-linguaray";

export default async function command(): Promise<void> {
  await openLinguaRay("translate", await getSelectedText());
}
