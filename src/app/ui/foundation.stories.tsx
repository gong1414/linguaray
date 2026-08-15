import type { Meta, StoryObj } from "@storybook/react-vite";
import {
  Alert,
  Badge,
  Button,
  Stack,
  Switch,
  Text,
  TextInput,
  Title,
} from "@mantine/core";

/**
 * Foundation — the Mantine components every page builds on, rendered with the
 * LinguaRay theme. This is the reference for token mapping (docs/UI-RULES.md):
 * if a component looks off here, fix the THEME, not the page.
 */
const meta: Meta = {
  title: "Foundation/Themed controls",
};
export default meta;

export const Controls: StoryObj = {
  render: () => (
    <Stack w={360}>
      <Title order={2}>LinguaRay 控件基准 / Control foundation</Title>
      <Text size="sm" c="dimmed">
        中文说明文本与 English secondary text both inherit the theme font stack.
      </Text>
      <Button>主操作 Primary</Button>
      <Button variant="light">次操作 Light</Button>
      <Button variant="subtle">幽灵 Subtle</Button>
      <Button variant="outline" color="danger">
        危险操作 Destructive
      </Button>
      <TextInput label="API Key" placeholder="sk-…" withAsterisk />
      <Switch label="启用历史记录 Enable history" defaultChecked />
      <Alert color="warning" title="需要辅助功能权限">
        请在系统设置中为 LinguaRay 开启辅助功能。Long Chinese wrapping is
        verified here so alerts never clip or overflow narrow windows.
      </Alert>
      <Badge color="success" variant="light">
        已连接 Connected
      </Badge>
      <Badge color="warning" variant="light">
        测试中 Testing
      </Badge>
      <Badge color="danger" variant="light">
        失败 Failed
      </Badge>
    </Stack>
  ),
};
