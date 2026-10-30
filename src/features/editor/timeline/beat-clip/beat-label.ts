// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";

import type { BeatWhat } from "../../island/beat/beat-island";

/** A beat on the strip: what happens at it, as its island says, or a moment
 * flagged while recording, named by its kind, which the user chooses, and
 * drawn in the kind's colour, `#rrggbb`. */
export type TimelineBeat =
  BeatWhat | { color: string; kind: "moment"; name: string };

/** What happens at a beat, said in a line: its clip's name and title. */
export function beatLabel(beat: TimelineBeat) {
  switch (beat.kind) {
    case "click":
      return t("editor-island-beat-click", { label: beat.label });
    case "type":
      return t("editor-island-beat-type", { label: beat.label });
    case "keys":
      return t("editor-island-beat-press", { keys: beat.keys.join("+") });
    case "switch":
      return t("editor-island-beat-switch", { app: beat.app });
    case "screen":
      return t("editor-island-beat-screen");
    case "sound":
      return t("editor-island-beat-sound");
    case "speech":
      return beat.line;
    case "moment":
      return t("editor-timeline-moment", { moment: beat.name });
  }
}
