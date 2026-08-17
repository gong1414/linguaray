import type { Meta, StoryObj } from "@storybook/react-vite";
import { Bubble, Sender } from "@ant-design/x";
import { Alert, Button, Form, Input, Switch, Tag, Typography } from "antd";
import { useUiStyles } from "./styles";

const meta: Meta = { title: "Foundation/Ant Design X controls" };
export default meta;

function ControlsStory() {
  const styles = useUiStyles();
  return (
    <div className={styles.page} style={{ width: 420 }}>
      <Typography.Title level={3} className={styles.title}>LinguaRay 控件基准 / Control foundation</Typography.Title>
      <Typography.Text type="secondary">中文说明文本与 English secondary text inherit Ant Design typography.</Typography.Text>
      <Button type="primary">主操作 Primary</Button>
      <Button>次操作 Secondary</Button>
      <Button type="text">幽灵 Text</Button>
      <Form.Item label="API Key" required><Input placeholder="sk-…" /></Form.Item>
      <Switch checkedChildren="开" unCheckedChildren="关" defaultChecked />
      <Alert type="warning" showIcon title="需要辅助功能权限" description="请在系统设置中为 LinguaRay 开启辅助功能。Long Chinese wrapping is verified here." />
      <div className={styles.rowWrap}><Tag color="success">已连接 Connected</Tag><Tag color="warning">测试中 Testing</Tag><Tag color="error">失败 Failed</Tag></div>
      <Bubble content="Ant Design X translation result" header="Provider" variant="outlined" />
      <Sender placeholder="输入要翻译的内容…" />
    </div>
  );
}

export const Controls: StoryObj = { render: () => <ControlsStory /> };
