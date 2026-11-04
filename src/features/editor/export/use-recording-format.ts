// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { clearExportFailure } from "./export-island-store";
import { recordingFormatPayload } from "./recording-format";

import type { RecordingFormat } from "../../../bindings/RecordingFormat";

/** The format the open recording is exported in, how to choose another,
 * and what the save is told of it. The choice holds for that recording
 * alone: each starts out as its video, since the sound alone is asked for
 * one recording, not the next. */
export function useRecordingFormat(artifactId: number | undefined) {
  const [chosen, setChosen] = useState<{
    artifactId: number;
    format: RecordingFormat;
  } | null>(null);
  const format: RecordingFormat =
    chosen && chosen.artifactId === artifactId ? chosen.format : "mp4";
  const choose = (next: RecordingFormat) => {
    if (artifactId === undefined) return;
    setChosen({ artifactId, format: next });
    clearExportFailure();
  };
  return [format, choose, () => recordingFormatPayload(format)] as const;
}
