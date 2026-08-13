import { type Component } from "solid-js";
// rev-8-2: the `@app` alias (configured in apps/ui-lab/vite.config.ts:16,
// vitest.config.ts:19, tsconfig.json:21 -> ../../src) resolves production src.
import {
  KeystoreRecoveryView,
  type KeystoreRecoveryViewProps,
} from "@app/features/settings/KeystoreRecovery";
import "./KeystoreRecovery.css";

export type KeystoreState = "healthy" | "corrupt";

export type KeystoreRecoveryProps = {
  state: KeystoreState;
};

const KeystoreRecovery: Component<KeystoreRecoveryProps> = (props) => {
  // rev-7-4: canned props for the COMPLETE production View (Banner + Confirm +
  // Toast + busy). No IPC — the lab is a pure renderer.
  const viewProps: KeystoreRecoveryViewProps = {
    state: props.state,
    reason: props.state === "corrupt" ? "Keystore unlock failed (lab fixture)" : "",
    resetOpen: false,
    busy: null,
    toasts: [],
    onArchive: () => {},
    onReset: () => {},
    onOpenReset: () => {},
    onCloseReset: () => {},
    onDismissToast: () => {},
  };
  return (
    <div class="keystore-shell" data-testid="lab-root">
      <KeystoreRecoveryView {...viewProps} />
    </div>
  );
};

export default KeystoreRecovery;
