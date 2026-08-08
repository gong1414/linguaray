# Surface 06: Keystore Recovery

**Surface ID:** `surface.keystore-recovery`
**Penpot 页面:** 20 Provider & Settings
**Penpot 画板尺寸:** [待查询]
**生产窗口默认尺寸:** 设置子页
**生产窗口最小尺寸:** 同设置（600×400）
**尺寸冲突说明:** 以生产窗口尺寸为实施标准

## 状态矩阵

| 状态 | 描述 | 组件 |
|---|---|---|
| Healthy | 无 banner；设置正常 | — |
| Corrupt | 错误 banner："密钥库不可读：{reason}" + "归档并重新输入" + "重置" | Banner (destructive) + Button × 2 |
| Archived | banner 清除；"请重新输入您的密钥"提示 | Banner (info/success) |
| Reset (confirm) | 警告对话框："历史将无法解密。继续？" | Confirm dialog (destructive) |

## 本地化 copy key

| key | en | zh |
|---|---|---|
| `keystore.healthy` | Keystore healthy | 密钥库正常 |
| `keystore.corrupt.title` | Keystore unreadable | 密钥库不可读 |
| `keystore.corrupt.description` | Keystore unreadable: {reason} | 密钥库不可读：{reason} |
| `keystore.corrupt.archive` | Archive & re-enter | 归档并重新输入 |
| `keystore.corrupt.reset` | Reset | 重置 |
| `keystore.archived.title` | Keys archived | 密钥已归档 |
| `keystore.archived.prompt` | Enter your keys again | 请重新输入您的密钥 |
| `keystore.reset.confirm.title` | Reset keystore? | 重置密钥库？ |
| `keystore.reset.confirm.message` | History will become undecryptable. Continue? | 历史将无法解密。继续？ |
| `keystore.reset.confirm.confirmLabel` | Reset | 重置 |
| `keystore.reset.confirm.cancelLabel` | Cancel | 取消 |

## 组件组合

- **顶部 banner：** Banner (destructive) 当 corrupt；Banner (info/success) 当 archived
  - 标题 + 描述（含 {reason}）
  - 操作按钮："归档并重新输入"（primary）/ "重置"（destructive）
- **重置确认：** Confirm dialog (destructive)
  - 初始焦点在 Cancel（防止 Enter 误触破坏性操作）
- **恢复后：** 跳转/引导到 Provider Center 重新输入密钥（链接 Surface 05）

## 页面特有约束

- 设置子页，遵循设置窗口尺寸与自适应规则。
- 重置时归档旧 keystore + DB 为 `.broken-*`，不删除。
- 重置警告历史/生词本将无法解密。
- 破坏性 Confirm 初始焦点在 Cancel。
