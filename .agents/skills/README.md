# LinguaRay project-local design skills

These skills apply only to this repository. No global skill directory or
Codex installation is changed.

| Skill | Upstream | Installed skill version |
| --- | --- | --- |
| Hallmark | https://github.com/nutlope/hallmark/tree/main/skills/hallmark | 1.1.0 |
| Impeccable | https://github.com/pbakaus/impeccable/tree/main/.agents/skills/impeccable | 4.3.1 |

Installed 2026-09-11. Each directory retains its upstream license; Impeccable
also retains NOTICE.md. Runtime binaries and review screenshots are ignored.

In a project task, use `$hallmark` for redesign/audit and `$impeccable` for
craft/review. Newly installed skills are discoverable on the next turn.
Read `PRODUCT.md` and `DESIGN.md` before applying either skill.

For the optional Impeccable engine, use `sh scripts/impeccable.sh <command>`
on macOS or `scripts\impeccable.cmd <command>` on Windows. These launchers
keep engine downloads under `.impeccable/runtime/` in this checkout.
No automatic hooks were installed. Flutter UI is evaluated using Widgetbook
and native tests; the Impeccable HTML/CSS detector is not a Dart validator.
