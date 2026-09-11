# Desktop redesign

Mode: Operate. Scope: all Flutter product surfaces, both desktop platforms.

First viewport: a complete grouped settings directory next to a clear page
heading, open content sections and aligned controls. Translation is a pair of
readable sheets, OCR a full-width editor. The user rejected navy; use the neutral
system in DESIGN.md. No new feature or product window is part of this redesign.

Signature interaction: settings destinations are directly reachable from the
grouped sidebar; translation retains its compact/expanded reading adaptation.
All existing callbacks and state semantics survive. Primary affordances use
strong neutral contrast; secondary actions recede until needed.

Validation: native widget tests, host-specific goldens and desktop build.
Impeccable's HTML/CSS detector cannot evaluate Flutter. Review rendered evidence.
