import {
  Badge,
  Button,
  Menu,
  MenuItem,
  MenuList,
  MenuPopover,
  MenuTrigger,
  Switch,
  Text,
} from "@fluentui/react-components";
import {
  AddRegular,
  ArrowDownRegular,
  ArrowReplyRegular,
  ArrowUpRegular,
  CopyRegular,
  DeleteRegular,
  LayerRegular,
  MoreHorizontalRegular,
  ServerRegular,
  StarRegular,
} from "@fluentui/react-icons";
import { SearchResultItem, SearchResultList } from "../../../ui/ueli";
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

/** Ueli result-list adapter for configured providers and provider presets. */
export function ProviderList(props: ProviderListProps) {
  const styles = useUiStyles();
  const t = props.t;
  const sorted = [...props.providers].sort((a, b) => a.sort_order - b.sort_order);
  const locked = props.exclusiveBusy;

  return (
    <div className={styles.stack} aria-label={t.providerListLabel} data-testid="provider-list">
      <div className={styles.rowBetween}>
        <Text weight="semibold">{t.providerListLabel}</Text>
        <Menu>
          <MenuTrigger disableButtonEnhancement>
            <Button appearance="primary" size="small" icon={<AddRegular />} disabled={locked}>
              {t.addProvider}
            </Button>
          </MenuTrigger>
          <MenuPopover style={{ maxHeight: 360, overflowY: "auto" }}>
            <MenuList>
              {props.presets.map((preset) => (
                <MenuItem
                  key={preset.templateId}
                  data-testid="preset-button"
                  title={preset.notes ?? undefined}
                  onClick={() => props.onAddPreset(preset)}
                >
                  <div className={styles.rowBetween} style={{ width: "100%" }}>
                    <span>{preset.name ?? "Ollama"}</span>
                    {preset.supportTier !== "ready" ? (
                      <Badge appearance="tint" color="warning">
                        {preset.supportTier === "setup_required" ? t.tier.setupRequired : t.tier.unverified}
                      </Badge>
                    ) : null}
                  </div>
                </MenuItem>
              ))}
            </MenuList>
          </MenuPopover>
        </Menu>
      </div>

      {props.providers.length === 0 ? (
        <div className={styles.empty} data-testid="provider-empty">
          <ServerRegular fontSize={28} aria-hidden />
          <Text weight="semibold">{t.empty.title}</Text>
          <Text size={300}>{t.empty.description}</Text>
        </div>
      ) : (
        <SearchResultList>
          {sorted.map((provider) => {
            const role = props.roleFor(provider.uuid);
            const rowDisabled = props.deletingUuid === provider.uuid || locked;
            const selected = props.selectedUuid === provider.uuid;
            const badges = (
              <div className={styles.rowWrap}>
                {!provider.enabled ? <Badge appearance="tint" color="subtle">{t.disabled}</Badge> : null}
                {provider.needs_key && !provider.hasKey ? <Badge appearance="tint" color="warning">{t.keyMissing}</Badge> : null}
                {role.kind === "primary" ? <Badge appearance="tint" color="success">{t.role.primary}</Badge> : null}
                {role.kind === "parallel" ? <Badge appearance="tint" color="informative">{t.role.parallel} {role.index}</Badge> : null}
                {role.kind === "fallback" ? <Badge appearance="tint" color="subtle">{t.role.fallback}</Badge> : null}
              </div>
            );

            const actions = (
              <div className={styles.row} onClick={(event) => event.stopPropagation()}>
                <Switch
                  aria-label={t.enabled}
                  checked={provider.enabled}
                  disabled={rowDisabled}
                  onChange={(_, data) => props.onToggle(provider.uuid, data.checked)}
                />
                <Menu>
                  <MenuTrigger disableButtonEnhancement>
                    <Button appearance="subtle" size="small" icon={<MoreHorizontalRegular />} aria-label={t.cardEdit.replace("{name}", provider.name)} disabled={rowDisabled} />
                  </MenuTrigger>
                  <MenuPopover>
                    <MenuList>
                      {provider.enabled && role.kind !== "primary" ? <MenuItem icon={<StarRegular />} onClick={() => props.onSetPrimary(provider.uuid)}>{t.setPrimary}</MenuItem> : null}
                      {provider.enabled && role.kind === "parallel" ? <MenuItem icon={<LayerRegular />} onClick={() => props.onRemoveParallel(provider.uuid)}>{t.removeParallel}</MenuItem> : null}
                      {provider.enabled && role.kind !== "parallel" && role.kind !== "primary" ? <MenuItem icon={<LayerRegular />} onClick={() => props.onAddParallel(provider.uuid)}>{t.addParallel}</MenuItem> : null}
                      {provider.enabled && role.kind !== "fallback" && role.kind !== "primary" ? <MenuItem icon={<ArrowReplyRegular />} onClick={() => props.onSetFallback(provider.uuid)}>{t.setFallback}</MenuItem> : null}
                      <MenuItem icon={<CopyRegular />} onClick={() => props.onDuplicate(provider.uuid)}>{t.duplicate}</MenuItem>
                      <MenuItem icon={<ArrowUpRegular />} onClick={() => props.onMoveUp(provider.uuid)}>{t.moveUp}</MenuItem>
                      <MenuItem icon={<ArrowDownRegular />} onClick={() => props.onMoveDown(provider.uuid)}>{t.moveDown}</MenuItem>
                      <MenuItem icon={<DeleteRegular />} onClick={() => props.onDelete(provider.uuid)}>{t.cardDelete.replace("{name}", provider.name)}</MenuItem>
                    </MenuList>
                  </MenuPopover>
                </Menu>
              </div>
            );

            return (
              <div
                key={provider.uuid}
                data-status={props.deletingUuid === provider.uuid ? "deleting" : provider.status}
                data-selected={selected || undefined}
              >
                <SearchResultItem
                  selected={selected}
                  icon={<ServerRegular fontSize={24} />}
                  name={provider.name}
                  details={badges}
                  actions={actions}
                  onClick={() => props.onEdit(provider.uuid)}
                />
              </div>
            );
          })}
        </SearchResultList>
      )}
    </div>
  );
}

export default ProviderList;
