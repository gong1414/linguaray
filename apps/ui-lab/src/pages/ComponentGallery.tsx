import { Show, createSignal, type Component } from "solid-js";
import {
  Button,
  IconButton,
  SegmentedControl,
  ShortcutChip,
  TextField,
  Select,
  Switch,
  StatusBadge,
  InlineError,
  Toast,
  Confirm,
  Spinner,
  EmptyState,
  TranslationCard,
  ResultCard,
  ProviderRow,
  ProviderCard,
  HistoryRow,
  SidebarItem,
  WindowChrome,
} from "@linguaray/ui";
import {
  Languages,
  Copy,
  Pin,
  Star,
  AlertTriangle,
  Server,
  Settings,
  Trash2,
} from "lucide-solid";
import type { Locale } from "../i18n";
import { galleryStrings } from "../i18n";
import "./ComponentGallery.css";

export type GalleryTheme = "light" | "dark";

type Props = {
  locale: Locale;
  theme: GalleryTheme;
};

/**
 * ComponentGallery — renders every R1 design component as a
 * `<section data-component-id="...">` showing its key state matrix. Used as
 * the visual-regression baseline surface and as an axe a11y regression
 * surface.
 *
 * Isolation rule: every section renders ONLY its target component's DOM.
 * Components that portal to <body> (Confirm/Dialog) are kept CLOSED by
 * default (open=false) so their overlays never escape into another
 * section's screenshot. The Confirm section exposes a trigger button that
 * opens the dialog on demand, but it starts closed.
 */
export const ComponentGallery: Component<Props> = (props) => {
  const t = () => galleryStrings[props.locale];

  return (
    <div class="gallery">
      <h1 class="gallery__title">{t().title}</h1>
      <div class="gallery__grid">
        {/* 1. Button */}
        <section class="gallery__section" data-component-id="button">
          <h2 class="gallery__section-title">{t().button.title}</h2>
          <div class="gallery__state">
            <span class="gallery__state-label">{t().variantsLabel}</span>
            <div class="gallery__state-row">
              <Button variant="primary">{t().button.primary}</Button>
              <Button variant="secondary">{t().button.secondary}</Button>
              <Button variant="ghost">{t().button.ghost}</Button>
              <Button variant="destructive">{t().button.destructive}</Button>
            </div>
          </div>
          <div class="gallery__state">
            <span class="gallery__state-label">{t().statesLabel}</span>
            <div class="gallery__state-row">
              <Button variant="primary" loading loadingLabel={t().button.loading}>
                {t().button.primary}
              </Button>
              <Button variant="secondary" disabled>
                {t().button.disabled}
              </Button>
            </div>
          </div>
        </section>

        {/* 2. IconButton */}
        <section class="gallery__section" data-component-id="icon-button">
          <h2 class="gallery__section-title">{t().iconButton.title}</h2>
          <div class="gallery__state">
            <span class="gallery__state-label">{t().variantsLabel}</span>
            <div class="gallery__state-row">
              <IconButton variant="ghost" aria-label={t().iconButton.copy}>
                <Copy size={16} />
              </IconButton>
              <IconButton variant="primary" aria-label={t().iconButton.translate}>
                <Languages size={16} />
              </IconButton>
              <IconButton variant="destructive" aria-label={t().iconButton.delete}>
                <Trash2 size={16} />
              </IconButton>
              <IconButton variant="ghost" aria-label={t().iconButton.delete} disabled>
                <Trash2 size={16} />
              </IconButton>
            </div>
          </div>
        </section>

        {/* 3. SegmentedControl */}
        <section class="gallery__section" data-component-id="segmented-control">
          <h2 class="gallery__section-title">{t().segmentedControl.title}</h2>
          <SegmentedControlTest labels={t().segmentedControl} />
        </section>

        {/* 4. ShortcutChip */}
        <section class="gallery__section" data-component-id="shortcut-chip">
          <h2 class="gallery__section-title">{t().shortcutChip.title}</h2>
          <div class="gallery__state">
            <span class="gallery__state-label">{t().statesLabel}</span>
            <div class="gallery__row-cluster">
              <ShortcutChip
                shortcut="⌘⇧C"
                status="clear"
                labels={t().shortcutChip.labels}
              />
              <ShortcutChip
                shortcut="⌘⇧C"
                status="recording"
                labels={t().shortcutChip.labels}
              />
              <ShortcutChip
                shortcut="⌘⇧C"
                status="conflict"
                labels={t().shortcutChip.labels}
                onClear={() => {}}
              />
              <ShortcutChip
                shortcut="⌘⇧C"
                status="clear"
                labels={t().shortcutChip.labels}
                disabled
              />
            </div>
          </div>
        </section>

        {/* 5. TextField */}
        <section class="gallery__section" data-component-id="text-field">
          <h2 class="gallery__section-title">{t().textField.title}</h2>
          <div class="gallery__state">
            <TextField
              label={t().textField.apiKey}
              placeholder="sk-…"
              helperText={t().textField.helper}
            />
          </div>
          <div class="gallery__state">
            <TextField
              label={t().textField.apiKey}
              placeholder="sk-…"
              errorText={t().textField.error}
            />
          </div>
        </section>

        {/* 6. Select */}
        <section class="gallery__section" data-component-id="select">
          <h2 class="gallery__section-title">{t().select.title}</h2>
          <SelectTest labels={t().select} />
        </section>

        {/* 7. Switch */}
        <section class="gallery__section" data-component-id="switch">
          <h2 class="gallery__section-title">{t().switch.title}</h2>
          <div class="gallery__state">
            <span class="gallery__state-label">{t().statesLabel}</span>
            <div class="gallery__row-cluster">
              <SwitchTest initial={true} label={t().switch.on} />
              <SwitchTest initial={false} label={t().switch.off} />
              <Switch checked={true} onChange={() => {}} label={t().switch.disabled} disabled />
            </div>
          </div>
        </section>

        {/* 8. StatusBadge */}
        <section class="gallery__section" data-component-id="status-badge">
          <h2 class="gallery__section-title">{t().statusBadge.title}</h2>
          <div class="gallery__state">
            <span class="gallery__state-label">{t().variantsLabel}</span>
            <div class="gallery__swatch-row">
              <StatusBadge variant="success" dot>{t().statusBadge.success}</StatusBadge>
              <StatusBadge variant="warning" dot>{t().statusBadge.warning}</StatusBadge>
              <StatusBadge variant="danger" dot>{t().statusBadge.danger}</StatusBadge>
              <StatusBadge variant="info" dot>{t().statusBadge.info}</StatusBadge>
              <StatusBadge variant="neutral" dot>{t().statusBadge.neutral}</StatusBadge>
            </div>
          </div>
        </section>

        {/* 9. InlineError */}
        <section class="gallery__section" data-component-id="inline-error">
          <h2 class="gallery__section-title">{t().inlineError.title}</h2>
          <InlineError>{t().inlineError.network}</InlineError>
          <InlineError icon={<AlertTriangle size={14} />}>{t().inlineError.config}</InlineError>
        </section>

        {/* 10. Toast */}
        <section class="gallery__section" data-component-id="toast">
          <h2 class="gallery__section-title">{t().toast.title}</h2>
          <div class="gallery__row-cluster">
            <Toast variant="info" message={t().toast.info} onDismiss={() => {}} />
            <Toast variant="success" message={t().toast.success} onDismiss={() => {}} />
            <Toast variant="warning" message={t().toast.warning} onDismiss={() => {}} />
            <Toast variant="destructive" message={t().toast.destructive} onDismiss={() => {}} />
          </div>
        </section>

        {/* 11. Confirm */}
        <section class="gallery__section" data-component-id="confirm">
          <h2 class="gallery__section-title">{t().confirm.title}</h2>
          <ConfirmTest labels={t().confirm} />
        </section>

        {/* 12. EmptyState */}
        <section class="gallery__section" data-component-id="empty-state">
          <h2 class="gallery__section-title">{t().emptyState.title}</h2>
          <EmptyState
            icon={<Server size={32} />}
            title={t().emptyState.title2}
            description={t().emptyState.desc}
            action={<Button variant="primary">{t().emptyState.action}</Button>}
          />
        </section>

        {/* 13. TranslationCard */}
        <section class="gallery__section" data-component-id="translation-card">
          <h2 class="gallery__section-title">{t().translationCard.title}</h2>
          <TranslationCard
            engineId="deepseek"
            engineLabel="DeepSeek"
            state={{ kind: "success", text: t().translationCard.result, elapsedMs: 120 }}
            labels={{
              loadingLabel: t().translationCard.loading,
              failureText: "Translation failed",
              retryLabel: "Retry",
            }}
          />
        </section>

        {/* 14. ResultCard */}
        <section class="gallery__section" data-component-id="result-card">
          <h2 class="gallery__section-title">{t().resultCard.title}</h2>
          <div class="gallery__row-cluster">
            <ResultCard
              engineId="openai"
              engineLabel="OpenAI"
              text={t().resultCard.success}
              elapsedMs={80}
              outcome="success"
              actions={[
                { label: t().resultCard.copy, icon: <Copy size={16} /> },
                { label: t().resultCard.pin, icon: <Pin size={16} /> },
              ]}
            />
            <ResultCard
              engineId="google"
              engineLabel="Google"
              outcome="failure"
              errorText={t().resultCard.failure}
            />
          </div>
        </section>

        {/* 15. ProviderRow */}
        <section class="gallery__section" data-component-id="provider-row">
          <h2 class="gallery__section-title">{t().providerRow.title}</h2>
          <ProviderRow
            name="OpenAI"
            template="gpt-4o"
            hasKey={true}
            needsKey={true}
            role={{ kind: "primary" }}
            enabled={true}
            labels={t().providerRow.labels}
            onToggle={() => {}}
            onEdit={() => {}}
            onDelete={() => {}}
          />
          {/* R12: keyless variant (needsKey=false) — neutral "Available" status,
              never "Key missing". */}
          <ProviderRow
            name="Ollama"
            template="llama3"
            hasKey={false}
            needsKey={false}
            role={{ kind: "none" }}
            enabled={true}
            labels={t().providerRow.labels}
            onToggle={() => {}}
            onEdit={() => {}}
            onDelete={() => {}}
          />
        </section>

        {/* 16. ProviderCard — R12.1: keyless variant (needsKey=false) shows
            "No key required", never "Key missing" or "Key saved". A
            needsKey=true + hasKey=true card is shown alongside for comparison.
            Labels default to defaultProviderCardLabels (English, matches the
            gallery's en locale). */}
        <section class="gallery__section" data-component-id="provider-card">
          <h2 class="gallery__section-title">Provider Card</h2>
          <ProviderCard
            profile={{ name: "Ollama", template: "llama3", status: "active" }}
            hasKey={false}
            needsKey={false}
            role={{ kind: "none" }}
            enabled={true}
            onToggle={() => {}}
            onEdit={() => {}}
            onDelete={() => {}}
          />
          <ProviderCard
            profile={{ name: "OpenAI", template: "gpt-4o", status: "active" }}
            hasKey={true}
            needsKey={true}
            role={{ kind: "primary" }}
            enabled={true}
            onToggle={() => {}}
            onEdit={() => {}}
            onDelete={() => {}}
          />
        </section>

        {/* 17. HistoryRow */}
        <section class="gallery__section" data-component-id="history-row">
          <h2 class="gallery__section-title">{t().historyRow.title}</h2>
          <HistoryRow
            sourceText={t().historyRow.source}
            resultPreview={t().historyRow.preview}
            timestamp={t().historyRow.time}
            engineLabel="DeepSeek"
            labels={{
              addFavorite: t().historyRow.addFav,
              removeFavorite: t().historyRow.removeFav,
            }}
            favorite={true}
            onToggleFavorite={() => {}}
          />
        </section>

        {/* 18. SidebarItem */}
        <section class="gallery__section" data-component-id="sidebar-item">
          <h2 class="gallery__section-title">{t().sidebarItem.title}</h2>
          <div class="gallery__row-cluster">
            <SidebarItem
              label={t().sidebarItem.settings}
              icon={<Settings size={16} />}
              active
            />
            <SidebarItem
              label={t().sidebarItem.history}
              icon={<Star size={16} />}
              badge="3"
            />
          </div>
        </section>

        {/* 19. Spinner — exercised under both full and reduced motion
            (reduced-motion overlay forces the text fallback via
            Spinner.css [data-motion="reduced"]). */}
        <section class="gallery__section" data-component-id="spinner">
          <h2 class="gallery__section-title">{t().spinner.title}</h2>
          <div class="gallery__state">
            <span class="gallery__state-label">{t().statesLabel}</span>
            <div class="gallery__row-cluster">
              <Spinner size={12} label={t().spinner.loading} />
              <Spinner size={16} label={t().spinner.loading} />
              <Spinner size={20} label={t().spinner.loading} />
            </div>
          </div>
        </section>

        {/* 20. WindowChrome */}
        <section class="gallery__section" data-component-id="window-chrome">
          <h2 class="gallery__section-title">{t().windowChrome.title}</h2>
          <WindowChrome
            title={t().windowChrome.title2}
            labels={t().windowChrome.labels}
            onClose={() => {}}
            onMinimize={() => {}}
          >
            <p>{t().windowChrome.body}</p>
          </WindowChrome>
        </section>

        {/* 21. Overflow · long CJK text — verifies Button/TextField/StatusBadge
            do not break layout when given unbroken long Chinese strings. */}
        <section class="gallery__section" data-component-id="overflow-cjk">
          <h2 class="gallery__section-title">{t().overflow.title}</h2>
          <div class="gallery__state">
            <span class="gallery__state-label">{t().button.title}</span>
            <Button variant="primary">{t().overflow.buttonLong}</Button>
          </div>
          <div class="gallery__state">
            <span class="gallery__state-label">{t().textField.title}</span>
            <TextField
              label={t().overflow.fieldLabel}
              placeholder={t().overflow.placeholder}
            />
          </div>
          <div class="gallery__state">
            <span class="gallery__state-label">{t().statusBadge.title}</span>
            <div class="gallery__swatch-row">
              <StatusBadge variant="info" dot>{t().overflow.badge}</StatusBadge>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
};

/* ---- isolated interactive sub-components (stateful) ----
 * Solid renders these inside the gallery; they own their own state so the
 * gallery does not need a top-level state manager. */

const SegmentedControlTest: Component<{
  labels: { title: string; en: string; zh: string };
}> = (props) => {
  const [val, setVal] = createSignal("en");
  return (
    <SegmentedControl
      ariaLabel={props.labels.title}
      value={val()}
      onChange={setVal}
      options={[
        { value: "en", label: props.labels.en },
        { value: "zh", label: props.labels.zh },
      ]}
    />
  );
};

const SelectTest: Component<{
  labels: { title: string; model: string; options: { value: string; label: string }[] };
}> = (props) => {
  const [val, setVal] = createSignal<string | null>("gpt-4o");
  return (
    <Select
      label={props.labels.model}
      value={val()}
      onChange={setVal}
      options={props.labels.options.map((o) => ({
        value: o.value,
        label: o.label,
        disabled: false,
      }))}
    />
  );
};

const SwitchTest: Component<{ initial: boolean; label: string }> = (props) => {
  const [checked, setChecked] = createSignal(props.initial);
  return <Switch checked={checked()} onChange={setChecked} label={props.label} />;
};

const ConfirmTest: Component<{
  labels: {
    title: string;
    primary: string;
    open: string;
    deleteTitle: string;
    deleteMsg: string;
    confirm: string;
    cancel: string;
  };
}> = (props) => {
  // CRITICAL: default closed. Confirm portals to <body>; an open dialog at
  // gallery mount would overlay every other section's screenshot. The
  // dialog is opened only when the trigger button is clicked.
  const [open, setOpen] = createSignal(false);
  return (
    <>
      <Button variant="primary" onClick={() => setOpen(true)}>
        {props.labels.open}
      </Button>
      <Show when={open()}>
        <Confirm
          open={open()}
          onOpenChange={setOpen}
          title={props.labels.deleteTitle}
          message={props.labels.deleteMsg}
          confirmLabel={props.labels.confirm}
          cancelLabel={props.labels.cancel}
          variant="destructive"
          onConfirm={() => setOpen(false)}
          onCancel={() => setOpen(false)}
        />
      </Show>
    </>
  );
};
