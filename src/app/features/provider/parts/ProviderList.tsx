import { ActionIcon, Badge, Button, Group, Paper, Stack, Switch, Text, Tooltip } from "@mantine/core";
import { ArrowDown, ArrowUp, Copy, CornerDownLeft, Layers, Plus, Server, Star } from "lucide-react";
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

/** Sidebar: provider rows (role actions + badges) + the preset grid. */
export function ProviderList(props: ProviderListProps) {
  const t = props.t;
  const sorted = [...props.providers].sort((a, b) => a.sort_order - b.sort_order);
  const locked = props.exclusiveBusy;

  return (
    <Stack gap="xs" aria-label={t.providerListLabel} data-testid="provider-list">
      <Text fw={600} size="sm">{t.addProvider}</Text>
      {props.providers.length === 0 ? (
        <Stack align="center" gap={4} py="lg" data-testid="provider-empty">
          <Server size={28} aria-hidden />
          <Text fw={500}>{t.empty.title}</Text>
          <Text size="sm" c="dimmed">{t.empty.description}</Text>
        </Stack>
      ) : (
        <Stack gap="xs">
          {sorted.map((p) => {
            const role = props.roleFor(p.uuid);
            const rowDisabled = props.deletingUuid === p.uuid || locked;
            return (
              <Paper
                key={p.uuid}
                withBorder
                p="xs"
                data-status={props.deletingUuid === p.uuid ? "deleting" : p.status}
                data-selected={props.selectedUuid === p.uuid || undefined}
              >
                <Group justify="space-between" wrap="nowrap" gap="xs">
                  <Group gap="xs" wrap="nowrap" style={{ minWidth: 0 }}>
                    <Switch
                      size="xs"
                      aria-label={t.enabled}
                      checked={p.enabled}
                      disabled={rowDisabled}
                      onChange={(e) => props.onToggle(p.uuid, e.currentTarget.checked)}
                    />
                    <Stack gap={2} style={{ minWidth: 0 }}>
                      <Button
                        variant={props.selectedUuid === p.uuid ? "light" : "subtle"}
                        size="compact-sm"
                        style={{ justifyContent: "flex-start" }}
                        aria-label={t.cardEdit.replace("{name}", p.name)}
                        disabled={rowDisabled}
                        onClick={() => props.onEdit(p.uuid)}
                      >
                        {p.name}
                      </Button>
                      <Group gap={4}>
                        {!p.enabled && (
                          <Badge variant="light" color="gray">{t.disabled}</Badge>
                        )}
                        {p.needs_key && !p.hasKey && (
                          <Badge variant="light" color="warning">{t.keyMissing}</Badge>
                        )}
                      </Group>
                    </Stack>
                  </Group>
                  <Group gap={4} wrap="nowrap">
                    {p.enabled && role.kind !== "primary" && (
                      <Tooltip label={t.setPrimary}>
                        <ActionIcon
                          variant="light"
                          size="sm"
                          aria-label={t.setPrimary}
                          disabled={rowDisabled}
                          onClick={() => props.onSetPrimary(p.uuid)}
                        >
                          <Star size={14} aria-hidden />
                        </ActionIcon>
                      </Tooltip>
                    )}
                    {p.enabled && role.kind === "parallel" && (
                      <Tooltip label={t.removeParallel}>
                        <ActionIcon
                          variant="light"
                          size="sm"
                          aria-label={t.removeParallel}
                          disabled={rowDisabled}
                          onClick={() => props.onRemoveParallel(p.uuid)}
                        >
                          <Layers size={14} aria-hidden />
                        </ActionIcon>
                      </Tooltip>
                    )}
                    {p.enabled && role.kind !== "parallel" && role.kind !== "primary" && (
                      <Tooltip label={t.addParallel}>
                        <ActionIcon
                          variant="light"
                          size="sm"
                          aria-label={t.addParallel}
                          disabled={rowDisabled}
                          onClick={() => props.onAddParallel(p.uuid)}
                        >
                          <Layers size={14} aria-hidden />
                        </ActionIcon>
                      </Tooltip>
                    )}
                    {p.enabled && role.kind !== "fallback" && role.kind !== "primary" && (
                      <Tooltip label={t.setFallback}>
                        <ActionIcon
                          variant="light"
                          size="sm"
                          aria-label={t.setFallback}
                          disabled={rowDisabled}
                          onClick={() => props.onSetFallback(p.uuid)}
                        >
                          <CornerDownLeft size={14} aria-hidden />
                        </ActionIcon>
                      </Tooltip>
                    )}
                    <Tooltip label={t.duplicate}>
                      <ActionIcon
                        variant="subtle"
                        size="sm"
                        aria-label={t.duplicate}
                        disabled={rowDisabled}
                        onClick={() => props.onDuplicate(p.uuid)}
                      >
                        <Copy size={14} aria-hidden />
                      </ActionIcon>
                    </Tooltip>
                    <Tooltip label={t.moveUp}>
                      <ActionIcon
                        variant="subtle"
                        size="sm"
                        aria-label={t.moveUp}
                        disabled={rowDisabled}
                        onClick={() => props.onMoveUp(p.uuid)}
                      >
                        <ArrowUp size={14} aria-hidden />
                      </ActionIcon>
                    </Tooltip>
                    <Tooltip label={t.moveDown}>
                      <ActionIcon
                        variant="subtle"
                        size="sm"
                        aria-label={t.moveDown}
                        disabled={rowDisabled}
                        onClick={() => props.onMoveDown(p.uuid)}
                      >
                        <ArrowDown size={14} aria-hidden />
                      </ActionIcon>
                    </Tooltip>
                    <Tooltip label={t.cardDelete.replace("{name}", p.name)}>
                      <ActionIcon
                        variant="light"
                        color="danger"
                        size="sm"
                        aria-label={t.cardDelete.replace("{name}", p.name)}
                        disabled={rowDisabled}
                        onClick={() => props.onDelete(p.uuid)}
                      >
                        <Server size={14} aria-hidden style={{ transform: "rotate(45deg)" }} />
                      </ActionIcon>
                    </Tooltip>
                  </Group>
                </Group>
                {role.kind !== "none" && (
                  <Group gap={4} mt={6}>
                    {role.kind === "primary" && (
                      <Badge variant="light" color="success">{t.role.primary}</Badge>
                    )}
                    {role.kind === "parallel" && (
                      <Badge variant="light" color="info">{t.role.parallel} {role.index}</Badge>
                    )}
                    {role.kind === "fallback" && (
                      <Badge variant="light" color="gray">{t.role.fallback}</Badge>
                    )}
                  </Group>
                )}
              </Paper>
            );
          })}
        </Stack>
      )}

      <Group gap="xs" mt="xs">
        {props.presets.map((preset) => (
          <Button
            key={preset.templateId}
            variant="default"
            size="xs"
            leftSection={<Plus size={12} aria-hidden />}
            disabled={locked}
            title={preset.notes ?? undefined}
            data-testid="preset-button"
            onClick={() => props.onAddPreset(preset)}
          >
            {preset.name ?? "Ollama"}
            {preset.supportTier !== "ready" && (
              <Badge ml={4} size="xs" variant="light" color="warning">
                {preset.supportTier === "setup_required" ? t.tier.setupRequired : t.tier.unverified}
              </Badge>
            )}
          </Button>
        ))}
      </Group>
    </Stack>
  );
}

export default ProviderList;
