import { Tooltip as KobalteTooltip } from "@kobalte/core/tooltip";
import { type Component, type JSX, type ValidComponent } from "solid-js";
import "./Tooltip.css";

export type TooltipSide = "top" | "bottom" | "left" | "right";

export type TooltipProps = {
  content: string;
  children: JSX.Element;
  side?: TooltipSide;
  /** Render the trigger AS this component (polymorphic). Avoids wrapping in
   *  an extra button — the child component receives Kobante's ref, events,
   *  and ARIA directly. */
  as?: ValidComponent;
  /** Props to spread onto the trigger component (when `as` is used). */
  triggerProps?: Record<string, unknown>;
  class?: string;
};

/**
 * MASTER §7 Tooltip. bg-elevated + fg, text-sm, max-width 240px.
 * Shows on hover AND keyboard focus; Esc closes (Kobante default).
 *
 * When `as` is provided, Kobante renders the trigger as that component
 * (e.g. `as={Button}`), so there is exactly ONE interactive element — no
 * wrapper button nesting. When `as` is omitted, children are wrapped in
 * a `<span>` (non-interactive) for text/inline content.
 */
const Tooltip: Component<TooltipProps> = (props) => {
  return (
    <KobalteTooltip placement={props.side ?? "top"}>
      <KobalteTooltip.Trigger
        as={props.as ?? "span"}
        class={`lr-tooltip__trigger${props.class ? ` ${props.class}` : ""}`}
        {...(props.triggerProps ?? {})}
      >
        {props.children}
      </KobalteTooltip.Trigger>
      <KobalteTooltip.Portal>
        <KobalteTooltip.Content class="lr-tooltip__content">
          {props.content}
          <KobalteTooltip.Arrow class="lr-tooltip__arrow" />
        </KobalteTooltip.Content>
      </KobalteTooltip.Portal>
    </KobalteTooltip>
  );
};

export default Tooltip;
