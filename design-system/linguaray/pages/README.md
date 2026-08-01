# Page Overrides

This directory contains page-specific design overrides. Each file (e.g.
`settings.md`, `popup.md`) applies ONLY to that page and takes precedence over
`MASTER.md` for layout/composition/content decisions.

## What page files MAY override

- ✅ Layout structure (grid, flex direction, sidebar arrangement)
- ✅ Component composition (which components, in what order)
- ✅ Content guidance (copy, labels, empty-state text)
- ✅ Density adjustments for that page (tighter/looser spacing)

## What page files may NOT override

- ❌ Global color tokens (MASTER.md §1)
- ❌ Contrast and keyboard accessibility rules (MASTER.md §6, §10)
- ❌ Component contracts (MASTER.md §7)
- ❌ Motion / reduced-motion rules (MASTER.md §5)
- ❌ Window base behavior (MASTER.md §8)

Changing any of these requires editing `MASTER.md` first.
