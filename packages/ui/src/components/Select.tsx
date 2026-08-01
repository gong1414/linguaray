import { ChevronDown, Check } from "lucide-solid";
import { Select as KobalteSelect } from "@kobalte/core/select";
import { Show, type Component } from "solid-js";
import Spinner from "./Spinner";
import "./Select.css";

export type SelectOption = {
  value: string;
  label: string;
  disabled: boolean;
};

export type SelectProps = {
  label: string;
  value: string | null;
  options: SelectOption[];
  onChange: (value: string) => void;
  disabled?: boolean;
  placeholder?: string;
  /** Loading: trigger becomes disabled + aria-busy + spinner (§4.3 Loading models). */
  loading?: boolean;
  /** Accessible label for the loading spinner (i18n). Default: "Loading…". */
  loadingLabel?: string;
  /** When present, shows Select.ErrorMessage + validationState=invalid. */
  errorText?: string;
  /** Optional helper description below the trigger. */
  description?: string;
  class?: string;
};

/**
 * MASTER §7 Select. Wraps Kobalte Select (unstyled) with our tokens.
 *
 * Label/description/error association uses Kobante's own Select.Label /
 * Select.Description / Select.ErrorMessage — these auto-generate
 * aria-labelledby / aria-describedby / aria-errormessage on the trigger,
 * so we do NOT hand-write a `label for` here.
 *
 * Loading = trigger disabled + aria-busy + Spinner. "Model manual entry"
 * is NOT built into Select — the consuming page renders a public TextField.
 */
const Select: Component<SelectProps> = (props) => {
  const selectedOption = (): SelectOption | null => {
    return props.options.find((o) => o.value === props.value) ?? null;
  };

  const isInvalid = (): boolean => Boolean(props.errorText);
  const isDisabled = (): boolean =>
    (props.disabled ?? false) || (props.loading ?? false);

  return (
    <KobalteSelect<SelectOption>
      class={`lr-select lr-focusable${props.class ? ` ${props.class}` : ""}`}
      options={props.options}
      value={selectedOption()}
      onChange={(opt: SelectOption | null) => {
        if (opt) props.onChange(opt.value);
      }}
      disabled={isDisabled()}
      placeholder={props.placeholder}
      validationState={isInvalid() ? "invalid" : "valid"}
      optionValue="value"
      optionTextValue="label"
      optionDisabled="disabled"
      itemComponent={(itemProps) => (
        <KobalteSelect.Item
          item={itemProps.item}
          class="lr-select__item"
        >
          <KobalteSelect.ItemLabel>
            {itemProps.item.rawValue.label}
          </KobalteSelect.ItemLabel>
          <KobalteSelect.ItemIndicator class="lr-select__item-indicator">
            <Check size={14} aria-hidden="true" />
          </KobalteSelect.ItemIndicator>
        </KobalteSelect.Item>
      )}
    >
      <KobalteSelect.Label class="lr-select__label">
        {props.label}
      </KobalteSelect.Label>

      <KobalteSelect.Trigger
        class="lr-select__trigger"
        aria-busy={props.loading ? "true" : undefined}
      >
        <span
          class="lr-select__value"
          classList={{ "lr-select__value--placeholder": !selectedOption() }}
        >
          <Show when={selectedOption()} fallback={props.placeholder}>
            {selectedOption()?.label}
          </Show>
        </span>
        <Show
          when={props.loading}
          fallback={
            <ChevronDown class="lr-select__icon" size={16} aria-hidden="true" />
          }
        >
          <Spinner size={12} label={props.loadingLabel ?? "Loading…"} />
        </Show>
      </KobalteSelect.Trigger>

      <Show when={props.description}>
        <KobalteSelect.Description class="lr-select__description">
          {props.description}
        </KobalteSelect.Description>
      </Show>

      <Show when={props.errorText}>
        <KobalteSelect.ErrorMessage class="lr-select__error">
          {props.errorText}
        </KobalteSelect.ErrorMessage>
      </Show>

      <KobalteSelect.Portal>
        <KobalteSelect.Content class="lr-select__content">
          <KobalteSelect.Listbox class="lr-select__listbox" />
        </KobalteSelect.Content>
      </KobalteSelect.Portal>
    </KobalteSelect>
  );
};

export default Select;
