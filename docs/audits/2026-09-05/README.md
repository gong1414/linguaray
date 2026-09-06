# 检测复现附件

对应检测基线 fa398c1。只使用本地 HTTP、内存模拟和临时运行时数据，不需要真实 API Key。

在仓库根目录，使用项目规定的 Flutter/Dart 与 Rust 工具链：

```sh
export PATH=/Users/daoyu/.cache/linguaray/flutter-3.47.1/bin:$PATH
python3 docs/audits/2026-09-05/run_probes.py
```

脚本用独占创建方式临时放置 Cargo 集成测试，结束后删除该测试文件；不会修改已有测试。基线代码上退出码 1 是预期结果：目录前缀检查失败，两个 Rust 行为断言失败。

- protocol_probe.rs：成功 SSE 的运行时回调与 UTF-8 跨分包断言，并打印当前提供商目录数量。
- detection_probe.dart：已显式选择语言时，检测 pending 期间翻译调用和结果事件均为 0；释放检测后翻译才开始。
- protocol_probe_output.txt：首次协议探针的实际输出。
- run_probes.py：组合运行目录生成器、协议和编排探针。

协议探针的 Runtime 使用独立临时目录。阻塞问题复现产生的后台线程随测试进程退出，不需要手工终止项目进程。
