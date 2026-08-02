import { Tooltip as KobanteTooltip } from "@kobalte/core/tooltip";
import { type Component, type JSX, type ValidComponent, createMemo } from "solid-js";
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
  /** Props to spread onto the trigger component (when `as` is used).
   *  A `class` here is MERGED with the base trigger class (not overridden). */
  triggerProps?: Record<string, unknown>;
  class?: string;
};

/**
 * MASTER §7 Tooltip. bg-elevated + fg, text-sm, max-width 240px.
 * Shows on hover AND keyboard focus; Esc closes (Kobante default).
 *
 * When `as` is provided, Kobante renders the trigger as that component
 * (e.g. `as={Button}` or `as="button"`), so there is exactly ONE interactive
 * element — no wrapper-button nesting. When `as` is omitted, children are
 * wrapped in a `<span>` (non-interactive) for text/inline content.
 *
 * `triggerProps.class` is merged with the base `lr-tooltip__trigger` class so
 * callers (e.g. the Settings rail) can add their own classes without losing the
 * tooltip styling hook. All other triggerProps flow straight through.
 */
const Tooltip: Component<TooltipProps> = (props) => {
  // Merged class is a memo so it stays reactive to prop changes. We read the
  // optional `class` from triggerProps defensively (it may be undefined).
  const triggerClass = createMemo(() => {
    const tpClass = props.triggerProps?.class;
    return [
      "lr-tooltip__trigger",
      props.class,
      typeof tpClass === "string" ? tpClass : undefined,
    ].filter(Boolean).join(" ");
  });

  // Build the spread bag minus `class` reactively, so Kobante's own props
  // (ref, handlers, aria-describedby) plus the caller's props all land on the
  // trigger. Using createMemo keeps the object identity stable across reads
  // unless the inputs actually change.
  const spreadProps = createMemo(() => {
    if (!props.triggerProps) return {};
    const out: Record<string, unknown> = {};
    for (const key of Object.keys(props.triggerProps)) {
      if (key !== "class") out[key] = props.triggerProps[key];
    }
    return out;
  });

  return (
    <KobanteTooltip placement={props.side ?? "top"}>
      <KobanteTooltip.Trigger
        as={props.as ?? "span"}
        class={triggerClass()}
        {...spreadProps()}
      >
        {props.children}
      </KobanteTooltip.Trigger>
      <KobanteTooltip.Portal>
        <KobanteTooltip.Content class="lr-tooltip__content">
          {props.content}
          <KobanteTooltip.Arrow class="lr-tooltip__arrow" />
        </KobanteTooltip.Content>
      </KobanteTooltip.Portal>
    </KobanteTooltip>
  );
};

export default Tooltip;
