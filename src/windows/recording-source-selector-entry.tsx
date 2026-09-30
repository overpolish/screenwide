// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingSourceSelectorWindow } from "../features/recording-sources/recording-source-selector-window";
import { RecordingSourceSync } from "../features/recording-sources/recording-source-sync";

/** The window, with the stores it reads kept in step with Rust and the other windows. */
export function RecordingSourceSelectorEntry() {
  return (
    <>
      <RecordingSourceSync />
      <RecordingSourceSelectorWindow />
    </>
  );
}
