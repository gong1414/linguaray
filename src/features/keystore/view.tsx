import {
  Button,
  Dialog,
  DialogActions,
  DialogBody,
  DialogContent,
  DialogSurface,
  DialogTitle,
  MessageBar,
  MessageBarActions,
  MessageBarBody,
  MessageBarTitle,
  Spinner,
  Text,
} from "@fluentui/react-components";
import { DismissRegular } from "@fluentui/react-icons";
import { detectLocale } from "../../app/i18n";
import { useUiStyles } from "../../ui/styles";
import { KEYSTORE_COPY } from "./copy";
import type { KeystoreController } from "./controller";

/** Pure presentational Keystore Recovery surface. */
export function KeystoreRecoveryView({ c }: { c: KeystoreController }) {
  const t = KEYSTORE_COPY[detectLocale()];
  const styles = useUiStyles();

  return (
    <section aria-label={t.pageTitle} data-testid="keystore-recovery" className={styles.page}>
      <Text as="h2" size={500} weight="semibold" className={styles.title}>{t.pageTitle}</Text>
      {c.state === "healthy" && (
        <MessageBar intent="success" data-testid="keystore-healthy">
          <MessageBarBody>{t.healthy}</MessageBarBody>
        </MessageBar>
      )}
      {c.state === "corrupt" && (
        <MessageBar intent="error" data-testid="keystore-corrupt">
          <MessageBarBody>
            <MessageBarTitle>{t.title}</MessageBarTitle>
            <Text size={300}>{t.description.replace("{reason}", c.reason)}</Text>
          </MessageBarBody>
          <MessageBarActions>
            <Button
              size="small"
              appearance="secondary"
              icon={c.busy === "archive" ? <Spinner size="tiny" /> : undefined}
              disabled={c.busy !== null}
              onClick={c.archive}
            >
              {t.archive}
            </Button>
            <Button
              size="small"
              appearance="primary"
              onClick={c.openReset}
              data-testid="keystore-reset-trigger"
            >
              {t.reset}
            </Button>
          </MessageBarActions>
        </MessageBar>
      )}

      {c.state === "archived" && (
        <MessageBar intent="info" data-testid="keystore-archived">
          <MessageBarBody>
            <MessageBarTitle>{t.archivedTitle}</MessageBarTitle>
            {t.archivedPrompt}
          </MessageBarBody>
        </MessageBar>
      )}

      <Dialog open={c.resetOpen} onOpenChange={(_, data) => !data.open && c.closeReset()}>
        <DialogSurface data-testid="keystore-reset-modal">
          <DialogBody>
            <DialogTitle>{t.resetConfirmTitle}</DialogTitle>
            <DialogContent><Text>{t.resetConfirmMessage}</Text></DialogContent>
            <DialogActions>
              <Button appearance="secondary" onClick={c.closeReset}>{t.resetConfirmCancelLabel}</Button>
              <Button
                appearance="primary"
                icon={c.busy === "reset" ? <Spinner size="tiny" /> : undefined}
                disabled={c.busy === "reset"}
                onClick={c.reset}
                data-testid="keystore-reset-confirm"
              >
                {t.resetConfirmConfirmLabel}
              </Button>
            </DialogActions>
          </DialogBody>
        </DialogSurface>
      </Dialog>

      {c.toasts.length > 0 && (
        <div className={styles.stack} aria-live="polite" data-testid="keystore-toasts">
          {c.toasts.map((entry) => (
            <MessageBar
              key={entry.id}
              intent={entry.variant === "destructive" ? "error" : entry.variant === "warning" ? "warning" : entry.variant === "success" ? "success" : "info"}
            >
              <MessageBarBody>{entry.message}</MessageBarBody>
              <MessageBarActions
                containerAction={
                  <Button
                    appearance="transparent"
                    icon={<DismissRegular />}
                    aria-label="Dismiss"
                    onClick={() => c.dismissToast(entry.id)}
                  />
                }
              />
            </MessageBar>
          ))}
        </div>
      )}
    </section>
  );
}

export default KeystoreRecoveryView;
