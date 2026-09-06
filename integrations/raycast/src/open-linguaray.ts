import { open, showHUD } from "@raycast/api";

export async function openLinguaRay(action: string, text?: string): Promise<void> {
  const query = text?.trim() ? `?text=${encodeURIComponent(text.trim())}` : "";
  await open(`linguaray://${action}${query}`);
  await showHUD("Sent to LinguaRay");
}
