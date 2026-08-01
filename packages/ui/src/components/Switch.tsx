import { Switch as KobalteSwitch } from "@kobalte/core/switch";
import { type Component } from "solid-js";
import "./Switch.css";

export type SwitchProps = {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  label: string;
  class?: string;
};

/**
 * MASTER §7 Switch. Controlled (checked + onChange callback).
 * On = primary-fill track + white thumb; Off = border-strong + white thumb
 * (same thumb color both states per §7).
 */
const Switch: Component<SwitchProps> = (props) => {
  return (
    <KobalteSwitch
      class={`lr-switch lr-focusable${props.disabled ? " lr-switch--disabled" : ""}${
        props.class ? ` ${props.class}` : ""
      }`}
      checked={props.checked}
      onChange={props.onChange}
      disabled={props.disabled ?? false}
    >
      <KobalteSwitch.Input />
      <KobalteSwitch.Control class="lr-switch__control">
        <KobalteSwitch.Thumb class="lr-switch__thumb" />
      </KobalteSwitch.Control>
      <KobalteSwitch.Label class="lr-switch__label">
        {props.label}
      </KobalteSwitch.Label>
    </KobalteSwitch>
  );
};

export default Switch;
