// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ChevronDown } from "lucide-react";

import { EditableTitle } from "../../../composites/editable-title/editable-title";
import { t } from "../../../i18n/i18n";
import { IconButton } from "../../../primitives/button/icon-button";

// A run of turns' speaker, beside their avatar and above what they said.
// The name is edited where it stands and renames them throughout the notes,
// as naming a speaker in a note taker does; the chevron opens who said this
// run, to give these words alone to someone else.

export function SpeakerLabel({
  isReadOnly,
  name,
  onChooseSpeaker,
  onRename,
}: {
  isReadOnly: boolean;
  name: string;
  /** Opens the speakers to give this run to, under `anchor`. */
  onChooseSpeaker: (anchor: DOMRect) => void;
  onRename: (name: string) => void;
}) {
  return (
    <div className="flex items-center gap-1">
      {isReadOnly ? (
        <span className="text-label text-fg" dir="auto">
          {name}
        </span>
      ) : (
        <EditableTitle
          className="text-label text-fg"
          label={t("notes-view-speaker-name", { speaker: name })}
          onChange={onRename}
          title={name}
        />
      )}
      {isReadOnly ? null : (
        <IconButton
          aria-label={t("notes-view-said-by", { speaker: name })}
          className="size-part-height text-fg-tertiary opacity-0 group-hover:opacity-100 focus-visible:opacity-100 enabled:hover:text-fg"
          onPress={(event) => {
            onChooseSpeaker(event.target.getBoundingClientRect());
          }}
          variant="ghost"
        >
          <ChevronDown aria-hidden className="size-icon-sm" />
        </IconButton>
      )}
    </div>
  );
}
