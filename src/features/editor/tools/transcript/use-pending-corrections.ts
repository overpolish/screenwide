// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo, useState } from "react";

import { TranscriptCorrection } from "../tool-panel-transcript";

import { correctionsWith, isCorrectionApplied } from "./transcript-units";
import { TranscriptWord } from "./transcript-words";

type Pending = {
  changes: TranscriptCorrection[];
  /** The editor's corrections the changes were last checked against. */
  published: readonly TranscriptCorrection[];
};

/**
 * The corrections as the panel shows them: the editor's, with each one typed
 * since laid over them until the editor publishes it. Each keystroke builds
 * on the text before it, so typing faster than the editor answers would
 * otherwise drop letters.
 */
export function usePendingCorrections(
  words: readonly TranscriptWord[],
  published: readonly TranscriptCorrection[],
) {
  const [pending, setPending] = useState<Pending>({ changes: [], published });
  let { changes } = pending;
  if (pending.published !== published) {
    changes = changes.filter(
      (change) => !isCorrectionApplied(words, published, change),
    );
    setPending({ changes, published });
  }
  const corrections = useMemo(
    () =>
      changes.reduce<readonly TranscriptCorrection[]>(
        (shown, change) => correctionsWith(words, shown, change),
        published,
      ),
    [changes, published, words],
  );
  const add = (change: TranscriptCorrection) => {
    setPending((current) => ({
      ...current,
      changes: [
        ...current.changes.filter(
          (other) =>
            other.endMs <= change.startMs || change.endMs <= other.startMs,
        ),
        change,
      ],
    }));
  };
  return { add, corrections };
}
