import type { ReactNode } from "react";
import { Actions } from "@ant-design/x";
import { Button, Tooltip } from "antd";

export type XAction = {
  key: string;
  label: string;
  icon: ReactNode;
  onClick: () => void;
  active?: boolean;
  danger?: boolean;
};

/** Accessible action buttons rendered through Ant Design X Actions. */
export function XActionBar({ actions }: { actions: XAction[] }) {
  return (
    <Actions
      items={actions.map((action) => ({
        key: action.key,
        label: action.label,
        danger: action.danger,
        actionRender: (
          <Tooltip title={action.label}>
            <Button
              type={action.active ? "primary" : "text"}
              size="small"
              danger={action.danger}
              icon={action.icon}
              aria-label={action.label}
              onClick={action.onClick}
            />
          </Tooltip>
        ),
      }))}
    />
  );
}
