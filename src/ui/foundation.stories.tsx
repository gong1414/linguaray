import type { Meta, StoryObj } from "@storybook/react-vite";
import {
  Badge,
  Button,
  Field,
  Input,
  MessageBar,
  MessageBarBody,
  MessageBarTitle,
  Switch,
  Text,
} from "@fluentui/react-components";
import { useUiStyles } from "./styles";

const meta: Meta = { title: "Foundation/Fluent controls" };
export default meta;

function ControlsStory() {
  const styles = useUiStyles();
  return (
    <div className={styles.page} style={{ width: 360 }}>
      <Text as="h2" size={500} weight="semibold" className={styles.title}>LinguaRay 控件基准 / Control foundation</Text>
      <Text size={300} className={styles.muted}>中文说明文本与 English secondary text both inherit Fluent typography.</Text>
      <Button appearance="primary">主操作 Primary</Button>
      <Button appearance="secondary">次操作 Secondary</Button>
      <Button appearance="subtle">幽灵 Subtle</Button>
      <Field label="API Key" required><Input placeholder="sk-…" /></Field>
      <Switch label="启用历史记录 Enable history" defaultChecked />
      <MessageBar intent="warning"><MessageBarBody><MessageBarTitle>需要辅助功能权限</MessageBarTitle>请在系统设置中为 LinguaRay 开启辅助功能。Long Chinese wrapping is verified here.</MessageBarBody></MessageBar>
      <div className={styles.rowWrap}>
        <Badge appearance="tint" color="success">已连接 Connected</Badge>
        <Badge appearance="tint" color="warning">测试中 Testing</Badge>
        <Badge appearance="tint" color="danger">失败 Failed</Badge>
      </div>
    </div>
  );
}

export const Controls: StoryObj = { render: () => <ControlsStory /> };
