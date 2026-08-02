import type { JSX, ParentComponent } from "solid-js";

/**
 * Renders children visually hidden but available to the assistive tree.
 * Used for accessible names on icon-only controls and the Spinner text fallback.
 */
const VisuallyHidden: ParentComponent<{ as?: keyof JSX.HTMLElementTags }> = (
  props,
) => {
  const Tag = (props.as ?? "span") as "span";
  return <Tag class="lr-visually-hidden">{props.children}</Tag>;
};

export default VisuallyHidden;
