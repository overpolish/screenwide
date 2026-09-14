<!-- SPDX-FileCopyrightText: 2026 overpolish -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Windows backlog

- Windows OSC: dragging a clip or screenshot can snap the visual back to the default even when the OSC reports the correct value and dimensions. Investigate the GPU compositor path.
- Glide cannot begin while an elevated (high-integrity) window such as Task Manager owns the foreground. Verified 2026-09-14 with a probe from a medium-integrity process: the low-level hooks, raw input sinks (`RIDEV_INPUTSINK`) and `GetAsyncKeyState` all read nothing until a normal window is focused; only `GetCursorPos` keeps moving, which cannot tell a glide from ordinary mouse use. The only fixes are a `uiAccess="true"` manifest (needs an Authenticode certificate and a Program Files install) or running elevated. Gliding an elevated window itself is refused up front with the lock feedback.
