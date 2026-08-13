import { type Component } from "solid-js";
import { Pencil, Trash2 } from "lucide-solid";
import Switch from "./Switch";
import StatusBadge from "./StatusBadge";
import { providerStatus } from "./providerPresentation";
import type { ProviderRole } from "./providerTypes";
import "./ProviderRow.css";

export type ProviderRowLabels = {
  edit: string;
  delete: string;
  enabled: string;
  /** 状态码 → 本地化文字映射（status code 不直接作文字）。 */
  statusText: Record<"active" | "available" | "key-missing" | "disabled", string>;
};

export type ProviderRowProps = {
  name: string;
  template: string;
  hasKey: boolean;
  /** R11: whether this provider type requires an API key. A keyless provider
   *  (`needs_key=false`) renders the neutral "available" status and never shows
   *  "key-missing", even when `hasKey` is false. */
  needsKey: boolean;
  role: ProviderRole;
  enabled: boolean;
  active?: boolean;
  /** When true, all row actions (toggle / edit / delete) are disabled. Used to
   *  lock the row while a delete is in-flight (prevents double-delete + races). */
  disabled?: boolean;
  labels: ProviderRowLabels;
  onToggle: (enabled: boolean) => void;
  onEdit: () => void;
  onDelete: () => void;
};

const ProviderRow: Component<ProviderRowProps> = (props) => {
  const status = () => providerStatus(props.role, props.hasKey, props.enabled, props.needsKey);
  return (
    <div
      class="provider-row"
      classList={{ "provider-row--active": !!props.active }}
    >
      <div class="provider-row__info">
        <span class="provider-row__name">{props.name}</span>
        <span class="provider-row__template">{props.template}</span>
      </div>
      <StatusBadge variant={status().variant} dot>{props.labels.statusText[status().code]}</StatusBadge>
      <Switch checked={props.enabled} onChange={props.onToggle} label={props.labels.enabled} disabled={props.disabled} />
      <button type="button" class="provider-row__btn" aria-label={props.labels.edit} onClick={props.onEdit} disabled={props.disabled}>
        <Pencil size={16} />
      </button>
      <button type="button" class="provider-row__btn" aria-label={props.labels.delete} onClick={props.onDelete} disabled={props.disabled}>
        <Trash2 size={16} />
      </button>
    </div>
  );
};
export default ProviderRow;
