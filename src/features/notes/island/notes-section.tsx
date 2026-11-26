// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { Alert } from "../../../primitives/alert/alert";
import { Button } from "../../../primitives/button/button";
import { IslandParts } from "../../../primitives/island/island-parts";
import { IslandWorkLine } from "../../editor/island/island-work-line";

import type { NotesParts } from "./notes-island";
import type { NotesProblem } from "../../../bindings/NotesProblem";

const problemMessage = (problem: NotesProblem) => {
  switch (problem) {
    case "noModel":
      return t("notes-island-no-model");
    case "noMicrophone":
      return t("notes-island-no-microphone");
    case "noMicrophoneConnected":
      return t("notes-island-no-microphone-connected");
    case "noSystemAudio":
      return t("notes-island-no-system-audio");
    case "notStarted":
      return t("notes-island-not-started");
    case "failed":
      return t("notes-island-failed");
  }
};

/** Whether Settings is where `problem` is put right. */
const isFixedInSettings = (problem: NotesProblem) => problem === "noModel";

/** What hangs under the notes' row: that the models are loading while
 * they are, or why the notes could not be taken, with Settings where that
 * is put right. Nothing otherwise, so the island has no section to make
 * room for. */
export function notesSection({ onAction, state }: NotesParts) {
  if (state.phase === "taking" && state.loading)
    return (
      <IslandParts>
        <IslandWorkLine
          key="loading"
          label={t("notes-island-loading-model")}
          value={null}
        />
      </IslandParts>
    );
  if (state.phase !== "problem") return undefined;
  return (
    <IslandParts>
      <div className="w-80" key={`problem-${state.problem}`}>
        <Alert
          action={
            isFixedInSettings(state.problem) ? (
              <Button
                onPress={() => {
                  onAction("openSettings");
                }}
              >
                {t("notes-island-open-settings")}
              </Button>
            ) : undefined
          }
          color="error"
          role="alert"
        >
          {problemMessage(state.problem)}
        </Alert>
      </div>
    </IslandParts>
  );
}
