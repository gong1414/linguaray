import { Alert, Button, Modal, Spin, Typography } from "antd";
import { CloseOutlined } from "@ant-design/icons";
import { detectLocale } from "../../app/i18n";
import { useUiStyles } from "../../ui/styles";
import { KEYSTORE_COPY } from "./copy";
import type { KeystoreController } from "./controller";

/** Pure presentational Ant Design Keystore Recovery surface. */
export function KeystoreRecoveryView({ c }: { c: KeystoreController }) {
  const t = KEYSTORE_COPY[detectLocale()];
  const styles = useUiStyles();

  return (
    <section aria-label={t.pageTitle} data-testid="keystore-recovery" className={styles.page}>
      <Typography.Title level={4} className={styles.title}>{t.pageTitle}</Typography.Title>
      {c.state === "healthy" ? <Alert type="success" showIcon title={t.healthy} data-testid="keystore-healthy" /> : null}
      {c.state === "corrupt" ? (
        <Alert
          type="error"
          showIcon
          data-testid="keystore-corrupt"
          title={t.title}
          description={t.description.replace("{reason}", c.reason)}
          action={
            <div className={styles.rowWrap}>
              <Button size="small" icon={c.busy === "archive" ? <Spin size="small" /> : undefined} disabled={c.busy !== null} onClick={c.archive}>{t.archive}</Button>
              <Button size="small" type="primary" danger onClick={c.openReset} data-testid="keystore-reset-trigger">{t.reset}</Button>
            </div>
          }
        />
      ) : null}
      {c.state === "archived" ? <Alert type="info" showIcon title={t.archivedTitle} description={t.archivedPrompt} data-testid="keystore-archived" /> : null}

      <Modal
        open={c.resetOpen}
        title={t.resetConfirmTitle}
        onCancel={c.closeReset}
        footer={[
          <Button key="cancel" onClick={c.closeReset}>{t.resetConfirmCancelLabel}</Button>,
          <Button key="confirm" type="primary" danger icon={c.busy === "reset" ? <Spin size="small" /> : undefined} disabled={c.busy === "reset"} onClick={c.reset} data-testid="keystore-reset-confirm">{t.resetConfirmConfirmLabel}</Button>,
        ]}
        data-testid="keystore-reset-modal"
      >
        <Typography.Paragraph>{t.resetConfirmMessage}</Typography.Paragraph>
      </Modal>

      {c.toasts.length > 0 ? (
        <div className={styles.stack} aria-live="polite" data-testid="keystore-toasts">
          {c.toasts.map((entry) => (
            <Alert
              key={entry.id}
              type={entry.variant === "destructive" ? "error" : entry.variant === "warning" ? "warning" : entry.variant === "success" ? "success" : "info"}
              showIcon
              title={entry.message}
              action={<Button type="text" size="small" icon={<CloseOutlined aria-hidden />} aria-label="Dismiss" onClick={() => c.dismissToast(entry.id)} />}
            />
          ))}
        </div>
      ) : null}
    </section>
  );
}

export default KeystoreRecoveryView;
