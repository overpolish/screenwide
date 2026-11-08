<!-- SPDX-FileCopyrightText: 2026 overpolish -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Local dependency patches

Keep a short entry here whenever adding or changing a vendored dependency patch. Cargo overrides live in [`../Cargo.toml`](../Cargo.toml) under `[patch.crates-io]`. Line numbers refer to the local vendored files; update them when a patch changes. Preserve each dependency's upstream license files.

## tract-linalg 0.21.4

- **Source:** The crates.io release of `tract-linalg` 0.21.4, unchanged apart from `build.rs` and a `[lints.rust]` table at the end of `Cargo.toml`, in `tract-linalg-0.21.4`. The transcriber's own manifest ([`../transcriber/Cargo.toml`](../transcriber/Cargo.toml)) patches it in, since the transcriber is a workspace of its own; it holds tract at 0.21.4 for DeepFilterNet.
- **Reason:** Its build script writes the ARM64 kernels in Microsoft assembler syntax whenever the target is MSVC, though MSVC has no MASM for ARM64. Windows ARM64 builds of the transcriber failed in `tract-linalg`'s build script.
- **Changes:** `build.rs` lines 16–23: `use_masm` leaves out `aarch64`, so the kernels stay GNU assembly. MSVC's `cl` cannot assemble those, so `scripts/prepare-transcriber.mjs` builds the Windows ARM64 transcriber's C, C++ and assembly with LLVM's `clang-cl`, which whisper.cpp needs on ARM anyway; LLVM must be on `PATH`, as it is on GitHub's Windows ARM64 runners. `Cargo.toml` lines 169–174 allow the `deprecated`, `mismatched_lifetime_syntaxes` and `unexpected_cfgs` lints: Cargo lints a path dependency as local code, so upstream's warnings, which crates.io would cap, showed in every transcriber build.
- **Validation:** Cross-checked `tract-linalg`'s build script for `aarch64-pc-windows-msvc` with `clang-cl` on Windows x64, and rebuilt the x64 transcriber with the vendored crate. `pnpm transcriber:check-clean` runs the speech enhancer on the built transcriber; the Release workflow runs it on both Windows runners, which proves the ARM64 kernels on real ARM64.
- **Upgrade/removal:** Drop the patch when DeepFilterNet moves to a tract release that builds for Windows ARM64; retest with the Release workflow's Windows ARM64 job.

## nokhwa-bindings-windows 0.4.6

- **Source:** The crates.io release of `nokhwa-bindings-windows` 0.4.6, in `nokhwa-bindings-windows-0.4.6`, unchanged apart from `src/lib.rs`. The release ships no license file; upstream's is Apache-2.0, as its `Cargo.toml` says.
- **Reason:** Its capture timestamp added a sample's Media Foundation time to the wall clock at the stream's start, taking that time to count from the start. A capture device's samples count from boot, so every camera frame was dated days ahead, and the recording fell back to when frames arrived: 30–33 ms of jitter, and the camera's delay left in.
- **Changes:** `src/lib.rs` lines 1171–1184: the timestamp is the wall clock now less the sample's age, `MFGetSystemTime()` less its time; lines 51–53 import `MFGetSystemTime`. The `stream_epoch` field the old sum needed is gone (formerly lines 414–417, 502, 1134–1136 and 1226).
- **Validation:** A three-minute screen and camera recording on Windows 11 (RTX 2070 SUPER, a 30 fps webcam delivering 32): every frame's timestamp came 31–32 ms before it arrived, so `camera_frame_clock` took the camera's own times, and the camera and screen files ended within one camera frame of each other.
- **Upgrade/removal:** Drop the patch when a nokhwa release dates Windows samples correctly; retest by recording with the camera and checking that `camera_frame_clock` takes the camera's timestamps (age under 2 s) rather than falling back to arrival time.

## Template for future patches

### Dependency name and version

- **Source:** Upstream release/commit and local directory.
- **Reason:** Bug or compatibility requirement motivating the patch.
- **Changes:** File paths, local line numbers, and a brief description of each change.
- **Validation:** Checks run and any manual reproduction used.
- **Upgrade/removal:** Upstream issue/PR if available, removal condition, and retest steps.
