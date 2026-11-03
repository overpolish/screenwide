<!-- SPDX-FileCopyrightText: 2026 overpolish -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Local dependency patches

Keep a short entry here whenever adding or changing a vendored dependency patch. Cargo overrides live in [`../Cargo.toml`](../Cargo.toml) under `[patch.crates-io]`. Line numbers refer to the local vendored files; update them when a patch changes. Preserve each dependency's upstream license files.

## Template for future patches

### Dependency name and version

- **Source:** Upstream release/commit and local directory.
- **Reason:** Bug or compatibility requirement motivating the patch.
- **Changes:** File paths, local line numbers, and a brief description of each change.
- **Validation:** Checks run and any manual reproduction used.
- **Upgrade/removal:** Upstream issue/PR if available, removal condition, and retest steps.
