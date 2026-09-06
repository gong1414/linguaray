import { openLinguaRay } from "./open-linguaray";

export default async function command(): Promise<void> {
  await openLinguaRay("input-translate");
}
