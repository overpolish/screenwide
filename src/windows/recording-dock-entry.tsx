// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingDockWindow } from "../features/recording-controls/components/recording-dock-window";
import { RecordingStateSync } from "../features/recording-controls/recording-state-sync";

/** The window, with the stores it reads kept in step with Rust and the other windows. */
export function RecordingDockEntry() {
  return (
    <>
      <RecordingStateSync />
      <RecordingDockWindow />
    </>
  );
}
