import { Avatar, Button, Dropdown, Empty, List, Switch, Tag, Typography } from "antd";
import type { MenuProps } from "antd";
import {
  ArrowDownOutlined,
  ArrowUpOutlined,
  CloudServerOutlined,
  CopyOutlined,
  DeleteOutlined,
  EllipsisOutlined,
  FallOutlined,
  PlusOutlined,
  StarOutlined,
  SwapOutlined,
} from "@ant-design/icons";
import { useUiStyles } from "../../../ui/styles";
import type { ProviderCopy } from "../copy";
import type { Preset, ProviderProfileFE, RoleState } from "../model";

export type ProviderListProps = {
  t: ProviderCopy;
  providers: ProviderProfileFE[];
  selectedUuid: string | null;
  exclusiveBusy: boolean;
  deletingUuid: string | null;
  presets: Preset[];
  roleFor: (uuid: string) => RoleState;
  onToggle: (uuid: string, enabled: boolean) => void;
  onEdit: (uuid: string) => void;
  onDelete: (uuid: string) => void;
  onSetPrimary: (uuid: string) => void;
  onAddParallel: (uuid: string) => void;
  onRemoveParallel: (uuid: string) => void;
  onSetFallback: (uuid: string) => void;
  onDuplicate: (uuid: string) => void;
  onMoveUp: (uuid: string) => void;
  onMoveDown: (uuid: string) => void;
  onAddPreset: (preset: Preset) => void;
};

/** Ant Design provider list with menu-driven presets and row actions. */
export function ProviderList(props: ProviderListProps) {
  const styles = useUiStyles();
  const t = props.t;
  const sorted = [...props.providers].sort((a, b) => a.sort_order - b.sort_order);
  const locked = props.exclusiveBusy;
  const presetItems: MenuProps["items"] = props.presets.map((preset) => ({
    key: preset.templateId,
    label: (
      <div className={styles.rowBetween} data-testid="preset-button" title={preset.notes ?? undefined} tabIndex={-1}>
        <span>{preset.name ?? "Ollama"}</span>
        {preset.supportTier !== "ready" ? <Tag color="warning">{preset.supportTier === "setup_required" ? t.tier.setupRequired : t.tier.unverified}</Tag> : null}
      </div>
    ),
    onClick: () => props.onAddPreset(preset),
  }));

  return (
    <div className={styles.stack} aria-label={t.providerListLabel} data-testid="provider-list">
      <div className={styles.rowBetween}>
        <Typography.Title level={4} className={styles.title}>{t.providerListLabel}</Typography.Title>
        <Dropdown menu={{ items: presetItems }} trigger={["click"]}>
          <Button type="primary" size="small" icon={<PlusOutlined aria-hidden />} disabled={locked}>{t.addProvider}</Button>
        </Dropdown>
      </div>
      {props.providers.length === 0 ? (
        <Empty image={<CloudServerOutlined aria-hidden />} description={<><Typography.Text strong>{t.empty.title}</Typography.Text><br /><Typography.Text type="secondary">{t.empty.description}</Typography.Text></>} data-testid="provider-empty" />
      ) : (
        <List
          className="lr-provider-list"
          dataSource={sorted}
          renderItem={(provider) => {
            const role = props.roleFor(provider.uuid);
            const rowDisabled = props.deletingUuid === provider.uuid || locked;
            const selected = props.selectedUuid === provider.uuid;
            const roleItems: MenuProps["items"] = [
              provider.enabled && role.kind !== "primary" ? { key: "primary", icon: <StarOutlined aria-hidden />, label: t.setPrimary, onClick: () => props.onSetPrimary(provider.uuid) } : null,
              provider.enabled && role.kind === "parallel" ? { key: "remove-parallel", icon: <SwapOutlined aria-hidden />, label: t.removeParallel, onClick: () => props.onRemoveParallel(provider.uuid) } : null,
              provider.enabled && role.kind !== "parallel" && role.kind !== "primary" ? { key: "parallel", icon: <SwapOutlined aria-hidden />, label: t.addParallel, onClick: () => props.onAddParallel(provider.uuid) } : null,
              provider.enabled && role.kind !== "fallback" && role.kind !== "primary" ? { key: "fallback", icon: <FallOutlined aria-hidden />, label: t.setFallback, onClick: () => props.onSetFallback(provider.uuid) } : null,
              { type: "divider" },
              { key: "duplicate", icon: <CopyOutlined aria-hidden />, label: t.duplicate, onClick: () => props.onDuplicate(provider.uuid) },
              { key: "up", icon: <ArrowUpOutlined aria-hidden />, label: t.moveUp, onClick: () => props.onMoveUp(provider.uuid) },
              { key: "down", icon: <ArrowDownOutlined aria-hidden />, label: t.moveDown, onClick: () => props.onMoveDown(provider.uuid) },
              { key: "delete", danger: true, icon: <DeleteOutlined aria-hidden />, label: t.cardDelete.replace("{name}", provider.name), onClick: () => props.onDelete(provider.uuid) },
            ];
            const tags = (
              <div className={styles.rowWrap}>
                {!provider.enabled ? <Tag>{t.disabled}</Tag> : null}
                {provider.needs_key && !provider.hasKey ? <Tag color="warning">{t.keyMissing}</Tag> : null}
                {role.kind === "primary" ? <Tag color="success">{t.role.primary}</Tag> : null}
                {role.kind === "parallel" ? <Tag color="processing">{t.role.parallel} {role.index}</Tag> : null}
                {role.kind === "fallback" ? <Tag>{t.role.fallback}</Tag> : null}
              </div>
            );
            return (
              <List.Item
                className={selected ? "lr-provider-row lr-provider-row-selected" : "lr-provider-row"}
                data-status={props.deletingUuid === provider.uuid ? "deleting" : provider.status}
                data-selected={selected || undefined}
                onClick={() => props.onEdit(provider.uuid)}
                actions={[
                  <Switch key="enabled" aria-label={t.enabled} checked={provider.enabled} disabled={rowDisabled} onClick={(_, event) => event.stopPropagation()} onChange={(checked) => props.onToggle(provider.uuid, checked)} />,
                  <Dropdown key="menu" menu={{ items: roleItems }} trigger={["click"]}>
                    <Button type="text" size="small" icon={<EllipsisOutlined aria-hidden />} aria-label={t.cardEdit.replace("{name}", provider.name)} disabled={rowDisabled} onClick={(event) => event.stopPropagation()} />
                  </Dropdown>,
                ]}
              >
                <List.Item.Meta avatar={<Avatar icon={<CloudServerOutlined aria-hidden />} />} title={provider.name} description={tags} />
              </List.Item>
            );
          }}
        />
      )}
    </div>
  );
}

export default ProviderList;
