// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Scissors } from "lucide-react";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../../i18n/i18n";
import { IslandKeyHint } from "../../../../primitives/island/island-key-hint";

import { BeatPlays } from "./beat-parts";
import { BeatTitle } from "./beat-title";

import type { EditorIslandContent } from "../editor-island";

// A beat on the strip, in hand: something that happens in the recording,
// as opposed to a lull. Its row says what happens, and its cut takes it and
// its time away, confirmed as every cut on the strip is. Under it, how it
// plays: as recorded, or at a speed of its own, given by dragging its end
// with ⌥, which the note at its foot says. A flagged moment is a beat with
// its own island, for its voice note.

/** What happens at a beat. `keys` are named as `Keyboard` knows them,
 * pressed together in order. */
export type BeatWhat =
  | { app: string; kind: "switch" }
  | { keys: string[]; kind: "keys" }
  /** `label` names what was clicked or typed, where the recording knows. */
  | { kind: "click" | "type"; label?: string }
  | { kind: "camera" }
  | { kind: "screen" }
  | { kind: "sound" }
  | { kind: "speech"; line: string };

export type BeatIslandProps = {
  beat: BeatWhat;
  id: string;
  onCut: () => void;
  /** Told to play the beat as recorded again, undoing its retime. */
  onPlayAsRecorded: () => void;
  /** How many times as fast as recorded it plays. */
  speed: number;
};

export function beatIsland({
  beat,
  id,
  onCut,
  onPlayAsRecorded,
  speed,
}: BeatIslandProps): EditorIslandContent {
  return {
    actions: (
      <ConfirmActionButton
        confirmLabel={t("editor-island-beat-confirm-cut")}
        icon={<Scissors />}
        label={t("editor-island-beat-cut")}
        onConfirm={onCut}
        variant="ghost"
      />
    ),
    // Its note keeps to the start, on the island's text column.
    anchor: "start",
    below: <BeatPlays onPlayAsRecorded={onPlayAsRecorded} speed={speed} />,
    key: `beat-${id}`,
    // The key that turns dragging the beat's end from a trim into a retime.
    section: (
      <IslandKeyHint
        binding="Alt"
        message={(key) => t("editor-island-beat-hint", { key })}
      />
    ),
    title: <BeatTitle beat={beat} />,
  };
}
