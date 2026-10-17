// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CircularProgress } from "../../../components/base/circular-progress/circular-progress";
import { t } from "../../../i18n/i18n";
import { formatBytes } from "../../editor/duration";

/**
 * How much of the recording is on disk, before the timer. The slot is as wide
 * as the size, so the gaps either side of it stay even. Fixed decimals and
 * tabular figures hold that width from one reading to the next; the pill
 * grows only when the size gains a digit or a unit, a handful of times in a
 * recording. It updates in place, unanimated: it changes on almost every
 * reading, and rolling digits there would pull the eye from the timer.
 */
export function RecordingDockSize({ bytes }: { bytes: number | null }) {
  return (
    <div className="flex text-body text-content-fg-secondary tabular-nums">
      {bytes !== null && bytes > 0 ? (
        formatBytes(bytes, { fixedDecimals: true })
      ) : (
        <CircularProgress
          aria-label={t("recording-controls-measuring-size")}
          isIndeterminate
          size="small"
        />
      )}
    </div>
  );
}
