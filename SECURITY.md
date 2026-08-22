# Security policy

## Supported versions

Security fixes are applied to the latest published release and the `main`
branch. Older builds may not receive backports.

## Reporting a vulnerability

Please report vulnerabilities through
[GitHub private vulnerability reporting](https://github.com/gong1414/linguaray/security/advisories/new).
Do not open a public issue and do not include real credentials, private text, or
sensitive screenshots in a proof of concept.

Include, when available:

- the affected version, commit, and operating system;
- a clear description of the impact and attack conditions;
- minimal reproduction steps or a safe proof of concept;
- any suggested mitigation.

The maintainer will acknowledge the report, investigate it, and coordinate a
fix and disclosure. Please allow time for a release before publishing details.

## Sensitive areas

Reports involving secure storage, clipboard restoration, permission handling,
provider credentials, logs, local configuration, or the Flutter/Rust FFI
boundary are especially valuable.
