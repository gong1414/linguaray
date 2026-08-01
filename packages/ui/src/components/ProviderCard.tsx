import { Show, Switch as FlowSwitch, Match, type Component } from "solid-js";
import { Check, AlertTriangle, Server, Pencil, Trash2 } from "lucide-solid";
import Switch from "./Switch";
import "./ProviderCard.css";

/**
 * Discriminated union — cannot represent illegal role overlap (primary +
 * fallback, etc.). The page computes this from a validated ActiveSelection.
 */
export type ProviderRole =
  | { kind: "none" }
  | { kind: "primary" }
  | { kind: "parallel"; index: number } // 1-based for display
  | { kind: "fallback" };

export type ProviderProfile = {
  name: string;
  template: string;
  status: "active" | "deleting" | "deleted";
};

/** Localizable labels for ProviderCard. The consuming page passes these from
 *  its i18n dictionary so no English is hardcoded in the component. */
export type ProviderCardLabels = {
  primary: string;
  parallel: string; // e.g. "Parallel" — index appended as "#N"
  fallback: string;
  keySaved: string;
  keyMissing: string;
  enabled: string;
  disabled: string;
  edit: string; // aria-label template, {name} substituted
  delete: string; // aria-label template, {name} substituted
};

export const defaultProviderCardLabels: ProviderCardLabels = {
  primary: "Primary",
  parallel: "Parallel",
  fallback: "Fallback",
  keySaved: "Key saved",
  keyMissing: "Key missing",
  enabled: "Enabled",
  disabled: "Disabled",
  edit: "Edit {name}",
  delete: "Delete {name}",
};

export type ProviderCardProps = {
  profile: ProviderProfile;
  hasKey: boolean;
  role: ProviderRole;
  enabled: boolean;
  onToggle: (enabled: boolean) => void;
  onEdit: () => void;
  onDelete: () => void;
  labels?: Partial<ProviderCardLabels>;
  class?: string;
};

/**
 * MASTER §7 ProviderCard.
 *
 * Renders as a NON-interactive <div> — Switch, Edit, Delete are sibling
 * buttons, never nested inside a card-level <button>. Active primary =
 * 3px selected-fg border-left. All display strings come from `labels` (no
 * hardcoded English).
 */
const ProviderCard: Component<ProviderCardProps> = (props) => {
  const role = () => props.role;
  const labels = (): ProviderCardLabels => ({
    ...defaultProviderCardLabels,
    ...props.labels,
  });
  const isPrimary = () => role().kind === "primary";
  const isDeleting = () => props.profile.status === "deleting";

  const editLabel = () => labels().edit.replace("{name}", props.profile.name);
  const deleteLabel = () => labels().delete.replace("{name}", props.profile.name);

  return (
    <div
      class={`lr-provider-card${isPrimary() ? " lr-provider-card--primary" : ""}${
        isDeleting() ? " lr-provider-card--deleting" : ""
      }${props.class ? ` ${props.class}` : ""}`}
      data-role={role().kind}
      data-template={props.profile.template}
    >
      <div class="lr-provider-card__header">
        <div class="lr-provider-card__info">
          <div class="lr-provider-card__name-row">
            <Server size={16} aria-hidden="true" />
            <span class="lr-provider-card__name">{props.profile.name}</span>
            <span class="lr-provider-card__template">
              {props.profile.template}
            </span>
          </div>

          {/* Role badges — Switch/Match narrows the discriminated union */}
          <Show when={role().kind !== "none"}>
            <div class="lr-provider-card__roles">
              <FlowSwitch>
                <Match when={role().kind === "primary"}>
                  <span class="lr-provider-card__role-badge lr-provider-card__role-badge--primary">
                    {labels().primary}
                  </span>
                </Match>
                <Match when={role().kind === "parallel"}>
                  <span class="lr-provider-card__role-badge lr-provider-card__role-badge--parallel">
                    {labels().parallel} #{(role() as { kind: "parallel"; index: number }).index}
                  </span>
                </Match>
                <Match when={role().kind === "fallback"}>
                  <span class="lr-provider-card__role-badge lr-provider-card__role-badge--fallback">
                    {labels().fallback}
                  </span>
                </Match>
              </FlowSwitch>
            </div>
          </Show>

          {/* Key status */}
          <Show
            when={props.hasKey}
            fallback={
              <span class="lr-provider-card__key-status lr-provider-card__key-status--missing">
                <AlertTriangle size={12} aria-hidden="true" />
                {labels().keyMissing}
              </span>
            }
          >
            <span class="lr-provider-card__key-status lr-provider-card__key-status--saved">
              <Check size={12} aria-hidden="true" />
              {labels().keySaved}
            </span>
          </Show>
        </div>
      </div>

      {/* Actions — sibling buttons, NOT nested in a card button */}
      <div class="lr-provider-card__actions">
        <div class="lr-provider-card__toggle">
          <Switch
            checked={props.enabled}
            onChange={props.onToggle}
            disabled={isDeleting()}
            label={props.enabled ? labels().enabled : labels().disabled}
          />
        </div>
        <button
          type="button"
          class="lr-icon-btn lr-focusable lr-icon-btn--ghost lr-icon-btn--sm"
          aria-label={editLabel()}
          disabled={isDeleting()}
          onClick={() => props.onEdit()}
        >
          <Pencil size={14} aria-hidden="true" />
        </button>
        <button
          type="button"
          class="lr-icon-btn lr-focusable lr-icon-btn--ghost lr-icon-btn--sm"
          aria-label={deleteLabel()}
          disabled={isDeleting()}
          onClick={() => props.onDelete()}
        >
          <Trash2 size={14} aria-hidden="true" />
        </button>
      </div>
    </div>
  );
};

export default ProviderCard;
