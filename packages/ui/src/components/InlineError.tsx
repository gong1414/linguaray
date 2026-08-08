import { type Component, type JSX } from "solid-js";
import { AlertTriangle } from "lucide-solid";
import "./InlineError.css";

export type InlineErrorProps = {
  children: JSX.Element;
  id?: string;
  icon?: JSX.Element;
};

const InlineError: Component<InlineErrorProps> = (props) => {
  return (
    <p class="inline-error" role="alert" id={props.id}>
      <span class="inline-error__icon" aria-hidden="true">
        {props.icon ?? <AlertTriangle size={14} />}
      </span>
      <span class="inline-error__text">{props.children}</span>
    </p>
  );
};
export default InlineError;
