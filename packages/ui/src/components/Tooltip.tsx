import { Tooltip as KobalteTooltip } from "@kobalte/core/tooltip";
import { type Component, type JSX } from "solid-js";
import "./Tooltip.css";

export type TooltipSide = "top" | "bottom" | "left" | "right";

export type TooltipProps = {
  content: string;
  children: JSX.Element;
  side?: TooltipSide;
  class?: string;
};

/**
 * MASTER §7 Tooltip. bg-elevated + fg, text-sm, max-width 240px.
 * Shows on hover AND keyboard focus; Esc closes (Kobante default).
 */
const Tooltip: Component<TooltipProps> = (props) => {
  return (
    <KobalteTooltip placement={props.side ?? "top"}>
      <KobalteTooltip.Trigger class="lr-tooltip__trigger">
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
