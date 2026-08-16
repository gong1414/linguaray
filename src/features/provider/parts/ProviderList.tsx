import {
  Badge,
  Button,
  Card,
  Switch,
  Text,
  Tooltip,
} from "@fluentui/react-components";
import {
  AddRegular,
  ArrowDownRegular,
  ArrowReplyRegular,
  ArrowUpRegular,
  CopyRegular,
  DeleteRegular,
  LayerRegular,
  ServerRegular,
  StarRegular,
} from "@fluentui/react-icons";
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

function IconAction({ label, disabled, icon, onClick }: { label: string; disabled: boolean; icon: React.ReactElement; onClick: () => void }) {
  return (
    <Tooltip content={label} relationship="label">
      <Button appearance="subtle" size="small" icon={icon} aria-label={label} disabled={disabled} onClick={onClick} />
    </Tooltip>
  );
}

/** Sidebar: provider rows (role actions + badges) + the preset grid. */
export function ProviderList(props: ProviderListProps) {
  const styles = useUiStyles();
  const t = props.t;
  const sorted = [...props.providers].sort((a, b) => a.sort_order - b.sort_order);
  const locked = props.exclusiveBusy;

  return (
    <div className={styles.stack} aria-label={t.providerListLabel} data-testid="provider-list">
      <Text weight="semibold">{t.addProvider}</Text>
      {props.providers.length === 0 ? (
        <div className={styles.empty} data-testid="provider-empty">
          <ServerRegular fontSize={28} aria-hidden />
          <Text weight="semibold">{t.empty.title}</Text>
          <Text size={300}>{t.empty.description}</Text>
        </div>
      ) : (
        <div className={styles.list}>
          {sorted.map((p) => {
            const role = props.roleFor(p.uuid);
            const rowDisabled = props.deletingUuid === p.uuid || locked;
            const selected = props.selectedUuid === p.uuid;
            return (
              <Card
                key={p.uuid}
                appearance="outline"
                size="small"
                className={selected ? styles.selectedCard : styles.card}
                data-status={props.deletingUuid === p.uuid ? "deleting" : p.status}
                data-selected={selected || undefined}
              >
                <div className={styles.stackTight}>
                  <div className={styles.rowBetween}>
                    <div className={styles.row}>
                      <Switch
                        aria-label={t.enabled}
                        checked={p.enabled}
                        disabled={rowDisabled}
                        onChange={(_, data) => props.onToggle(p.uuid, data.checked)}
                      />
                      <div className={styles.stackTight}>
                        <Button
                          appearance={selected ? "primary" : "subtle"}
                          size="small"
                          aria-label={t.cardEdit.replace("{name}", p.name)}
                          disabled={rowDisabled}
                          onClick={() => props.onEdit(p.uuid)}
                        >
                          {p.name}
                        </Button>
                        <div className={styles.rowWrap}>
                          {!p.enabled && <Badge appearance="tint" color="subtle">{t.disabled}</Badge>}
                          {p.needs_key && !p.hasKey && <Badge appearance="tint" color="warning">{t.keyMissing}</Badge>}
                        </div>
                      </div>
                    </div>
                    <div className={styles.rowWrap}>
                      {p.enabled && role.kind !== "primary" && <IconAction label={t.setPrimary} disabled={rowDisabled} icon={<StarRegular />} onClick={() => props.onSetPrimary(p.uuid)} />}
                      {p.enabled && role.kind === "parallel" && <IconAction label={t.removeParallel} disabled={rowDisabled} icon={<LayerRegular />} onClick={() => props.onRemoveParallel(p.uuid)} />}
                      {p.enabled && role.kind !== "parallel" && role.kind !== "primary" && <IconAction label={t.addParallel} disabled={rowDisabled} icon={<LayerRegular />} onClick={() => props.onAddParallel(p.uuid)} />}
                      {p.enabled && role.kind !== "fallback" && role.kind !== "primary" && <IconAction label={t.setFallback} disabled={rowDisabled} icon={<ArrowReplyRegular />} onClick={() => props.onSetFallback(p.uuid)} />}
                      <IconAction label={t.duplicate} disabled={rowDisabled} icon={<CopyRegular />} onClick={() => props.onDuplicate(p.uuid)} />
                      <IconAction label={t.moveUp} disabled={rowDisabled} icon={<ArrowUpRegular />} onClick={() => props.onMoveUp(p.uuid)} />
                      <IconAction label={t.moveDown} disabled={rowDisabled} icon={<ArrowDownRegular />} onClick={() => props.onMoveDown(p.uuid)} />
                      <IconAction label={t.cardDelete.replace("{name}", p.name)} disabled={rowDisabled} icon={<DeleteRegular />} onClick={() => props.onDelete(p.uuid)} />
                    </div>
                  </div>
                  {role.kind !== "none" && (
                    <div className={styles.rowWrap}>
                      {role.kind === "primary" && <Badge appearance="tint" color="success">{t.role.primary}</Badge>}
                      {role.kind === "parallel" && <Badge appearance="tint" color="informative">{t.role.parallel} {role.index}</Badge>}
                      {role.kind === "fallback" && <Badge appearance="tint" color="subtle">{t.role.fallback}</Badge>}
                    </div>
                  )}
                </div>
              </Card>
            );
          })}
        </div>
      )}

      <div className={styles.rowWrap}>
        {props.presets.map((preset) => (
          <Button
            key={preset.templateId}
            appearance="secondary"
            size="small"
            icon={<AddRegular />}
            disabled={locked}
            title={preset.notes ?? undefined}
            data-testid="preset-button"
            onClick={() => props.onAddPreset(preset)}
          >
            {preset.name ?? "Ollama"}
            {preset.supportTier !== "ready" && <Badge appearance="tint" color="warning">{preset.supportTier === "setup_required" ? t.tier.setupRequired : t.tier.unverified}</Badge>}
          </Button>
        ))}
      </div>
    </div>
  );
}

export default ProviderList;
