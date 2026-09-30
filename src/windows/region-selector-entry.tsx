// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingStateSync } from "../features/recording-controls/recording-state-sync";
import { RecordingSourceSync } from "../features/recording-sources/recording-source-sync";
import { RegionSelectorWindow } from "../features/region-selector/region-selector-window";

/** The window, with the stores it reads kept in step with Rust and the other windows. */
export function RegionSelectorEntry() {
  return (
    <>
      <RecordingSourceSync />
      <RecordingStateSync />
      <RegionSelectorWindow />
    </>
  );
}
