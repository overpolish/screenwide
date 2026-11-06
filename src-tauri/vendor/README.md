<!-- SPDX-FileCopyrightText: 2026 overpolish -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Local dependency patches

Keep a short entry here whenever adding or changing a vendored dependency patch. Cargo overrides live in [`../Cargo.toml`](../Cargo.toml) under `[patch.crates-io]`. Line numbers refer to the local vendored files; update them when a patch changes. Preserve each dependency's upstream license files.

## tract-linalg 0.21.4

- **Source:** The crates.io release of `tract-linalg` 0.21.4, unchanged apart from `build.rs`, in `tract-linalg-0.21.4`. The transcriber's own manifest ([`../transcriber/Cargo.toml`](../transcriber/Cargo.toml)) patches it in, since the transcriber is a workspace of its own; it holds tract at 0.21.4 for DeepFilterNet.
- **Reason:** Its build script writes the ARM64 kernels in Microsoft assembler syntax whenever the target is MSVC, though MSVC has no MASM for ARM64, and hands them to `cl`, which cannot assemble GNU-syntax ARM64. Windows ARM64 builds of the transcriber failed in `tract-linalg`'s build script.
- **Changes:** `build.rs` lines 16–33: `use_masm` leaves out `aarch64`, and `arm64_cc` builds with LLVM's `clang-cl` and `llvm-lib` on Windows (plain `cc::Build::new()` elsewhere). Line 76 (`ConfigForHalf::cc`, the fp16 kernels and their probe) and line 189 (`arm64simd`) use `arm64_cc`. Building the transcriber for Windows ARM64 needs LLVM on `PATH`; GitHub's Windows ARM64 runners have it.
- **Validation:** Cross-checked `tract-linalg`'s build script for `aarch64-pc-windows-msvc` on Windows x64. `pnpm transcriber:check-clean` runs the speech enhancer on the built transcriber, and the Release workflow runs it on both Windows runners, where it proves the ARM64 kernels on real ARM64.
- **Upgrade/removal:** Drop the patch when DeepFilterNet moves to a tract release that builds for Windows ARM64; retest with the Release workflow's Windows ARM64 job.

## Template for future patches

### Dependency name and version

- **Source:** Upstream release/commit and local directory.
- **Reason:** Bug or compatibility requirement motivating the patch.
- **Changes:** File paths, local line numbers, and a brief description of each change.
- **Validation:** Checks run and any manual reproduction used.
- **Upgrade/removal:** Upstream issue/PR if available, removal condition, and retest steps.
