// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ChevronDown, Trash2 } from "lucide-react";

import { EditableTitle } from "../../../composites/editable-title/editable-title";
import { t } from "../../../i18n/i18n";
import { IconButton } from "../../../primitives/button/icon-button";
import { voiceNumber } from "../speakers/use-voice-clips";

// A run of turns' speaker, beside their avatar and above what they said.
// The name is edited where it stands and renames them throughout the notes,
// as naming a speaker in a note taker does, a numbered name selected whole
// to type over; the chevron opens who said this run, to give these words
// alone to someone else, and the bin removes them.

export function SpeakerLabel({
  isReadOnly,
  name,
  onChooseSpeaker,
  onRemove,
  onRename,
}: {
  isReadOnly: boolean;
  name: string;
  /** Opens the speakers to give this run to, under `anchor`. */
  onChooseSpeaker: (anchor: DOMRect) => void;
  /** Removes this run, everything said in it. */
  onRemove: () => void;
  onRename: (name: string) => void;
}) {
  // Shown while the run is hovered or a button in it is focused, so a page
  // of names does not read as a page of buttons.
  const quiet =
    "text-fg-secondary opacity-0 group-hover:opacity-100 focus-visible:opacity-100 enabled:hover:text-fg";
  return (
    <div className="flex items-center gap-inline">
      {isReadOnly ? (
        <span className="text-label text-fg" dir="auto">
          {name}
        </span>
      ) : (
        <EditableTitle
          className="text-label text-fg"
          label={t("notes-view-speaker-name", { speaker: name })}
          onChange={onRename}
          selectsAllOnFocus={voiceNumber(name) !== null}
          title={name}
        />
      )}
      {isReadOnly ? null : (
        <>
          <IconButton
            aria-label={t("notes-view-said-by", { speaker: name })}
            className={quiet}
            onPress={(event) => {
              onChooseSpeaker(event.target.getBoundingClientRect());
            }}
            variant="ghost"
          >
            <ChevronDown aria-hidden />
          </IconButton>
          <IconButton
            aria-label={t("notes-view-remove-run", { speaker: name })}
            className={quiet}
            onPress={onRemove}
            variant="ghost"
          >
            <Trash2 aria-hidden />
          </IconButton>
        </>
      )}
    </div>
  );
}
