import { Check, AlertTriangle, Server } from "lucide-solid";
import { Show, Switch as FlowSwitch, Match, type Component } from "solid-js";
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

export type ProviderCardProps = {
  profile: ProviderProfile;
  hasKey: boolean;
  role: ProviderRole;
  enabled: boolean;
  onToggle: (enabled: boolean) => void;
  onEdit: () => void;
  onDelete: () => void;
  class?: string;
};

/**
 * MASTER §7 ProviderCard.
 *
 * Renders as a NON-interactive <div> — Switch, Edit, Delete are sibling
 * buttons, never nested inside a card-level <button> (MASTER §7 Card/ListRow:
 * no nested interactive elements). Active primary = 3px selected-fg border-left.
 */
const ProviderCard: Component<ProviderCardProps> = (props) => {
  const role = () => props.role;
  const isPrimary = () => role().kind === "primary";
  const isDeleting = () => props.profile.status === "deleting";

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
                    Primary
                  </span>
                </Match>
                <Match when={role().kind === "parallel"}>
                  <span class="lr-provider-card__role-badge lr-provider-card__role-badge--parallel">
                    Parallel #{(role() as { kind: "parallel"; index: number }).index}
                  </span>
                </Match>
                <Match when={role().kind === "fallback"}>
                  <span class="lr-provider-card__role-badge lr-provider-card__role-badge--fallback">
                    Fallback
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
                Key missing
              </span>
            }
          >
            <span class="lr-provider-card__key-status lr-provider-card__key-status--saved">
              <Check size={12} aria-hidden="true" />
              Key saved
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
            label={props.enabled ? "Enabled" : "Disabled"}
          />
        </div>
        <button
          type="button"
          class="lr-icon-btn lr-focusable lr-icon-btn--ghost lr-icon-btn--sm"
          aria-label={`Edit ${props.profile.name}`}
          disabled={isDeleting()}
          onClick={() => props.onEdit()}
        >
          <Server size={14} aria-hidden="true" />
        </button>
        <button
          type="button"
          class="lr-icon-btn lr-focusable lr-icon-btn--ghost lr-icon-btn--sm"
          aria-label={`Delete ${props.profile.name}`}
          disabled={isDeleting()}
          onClick={() => props.onDelete()}
        >
          <AlertTriangle size={14} aria-hidden="true" />
        </button>
      </div>
    </div>
  );
};

export default ProviderCard;
