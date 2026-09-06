# LinguaRay documentation

This directory contains the public technical documentation for LinguaRay.

## Start here

- [Architecture](ARCHITECTURE.md) explains the Flutter, application, platform,
  and Rust runtime boundaries.
- [Contributing](../CONTRIBUTING.md) covers development setup, code generation,
  required checks, and pull request expectations.
- [Security policy](../SECURITY.md) describes supported versions and private
  vulnerability reporting.
- [Support](../SUPPORT.md) directs questions, bug reports, and feature requests
  to the appropriate GitHub channel.
- [Brand assets](../assets/brand/linguaray/README.md) documents the canonical
  logo and generated desktop assets.
- [Refactor behavior baseline](refactor/BEHAVIOR_BASELINE.md),
  [public surfaces](refactor/PUBLIC_SURFACES.md), and
  [platform parity](refactor/PLATFORM_PARITY.md) define the compatibility gates
  for behavior-preserving modernization work.
- [Refactor passes](refactor/REFACTOR_PASSES.md) records the behavior and proof
  for each reviewable cleanup, the
  [validation record](refactor/VALIDATION.md) captures the acceptance evidence,
  while the
  [migration backlog](refactor/MIGRATION_BACKLOG.md) keeps API, dependency,
  storage, and architecture changes separate.

Temporary implementation plans and machine-specific migration logs are not
maintained as public project documentation.
