// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Scissors } from "lucide-react";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../../i18n/i18n";

import { BeatTitle } from "./beat-title";

import type { EditorIslandContent } from "../editor-island";

// A beat on the strip, in hand: something that happens in the recording,
// as opposed to a lull. Its row says what happens, and its cut takes it and
// its time away, confirmed as every cut on the strip is. A flagged moment
// is a beat with its own island, for its voice note.

/** What happens at a beat. `keys` are named as `Keyboard` knows them,
 * pressed together in order. */
export type BeatWhat =
  | { app: string; kind: "switch" }
  | { keys: string[]; kind: "keys" }
  | { kind: "click" | "type"; label: string }
  | { kind: "screen" }
  | { kind: "sound" }
  | { kind: "speech"; line: string };

export type BeatIslandProps = {
  beat: BeatWhat;
  id: string;
  onCut: () => void;
};

export function beatIsland({
  beat,
  id,
  onCut,
}: BeatIslandProps): EditorIslandContent {
  return {
    actions: (
      <ConfirmActionButton
        confirmLabel={t("editor-island-beat-confirm-cut")}
        icon={<Scissors aria-hidden />}
        label={t("editor-island-beat-cut")}
        onConfirm={onCut}
        variant="ghost"
      />
    ),
    key: `beat-${id}`,
    title: <BeatTitle beat={beat} />,
  };
}
