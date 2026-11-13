// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SearchX } from "lucide-react";
import { useFilter } from "react-aria";

import { t } from "../../../../i18n/i18n";
import { Alert } from "../../../../primitives/alert/alert";
import { guessedKind } from "../../sound-track/dropped-clips";
import {
  dragLibrarySound,
  renameSound,
  useSoundLibrary,
} from "../../sound-track/sound-library";

import { soundDragLook } from "./drag-look";
import { LibraryRow } from "./library-row";
import { useLibraryPreview } from "./use-library-preview";

import type { AudioClipKind } from "../../../../bindings/AudioClipKind";

// The library's songs or its sound effects (a sound 10 seconds or longer is
// a song), found by title. Each can be heard in place, from its start or
// from anywhere along its waveform, and renamed. Pressed, one is added to
// the timeline as the kind shown; dragged out, drawn as a chip of its
// waveform and title, it lands on the lane it is let go on, as a file from
// the Finder does, where the system lets a drag start. A song pressed with
// no room on the music track adds nothing; dragged over the full rail, the
// drag shows the drop refused.

/** What the island asks of the editor: a sound from the library, added as
 * `kind`. */
export type SoundAddition = { kind: AudioClipKind; name: string };

export function LibrarySoundList({
  alsoShown,
  hasSongRoom,
  kind,
  onAdd,
  query,
}: {
  /** Sounds shown whatever the search, by name. */
  alsoShown: ReadonlySet<string>;
  /** Whether a song added now would have room after the last song. */
  hasSongRoom: boolean;
  kind: AudioClipKind;
  onAdd: (addition: SoundAddition) => void;
  /** The words searched for in the titles. */
  query: string;
}) {
  const library = useSoundLibrary();
  const { contains } = useFilter({ sensitivity: "base" });
  const preview = useLibraryPreview();
  const ofKind = library.filter((sound) => guessedKind(sound) === kind);
  const shown = ofKind.filter(
    (sound) => alsoShown.has(sound.name) || contains(sound.title, query),
  );
  const isFull = kind === "music" && !hasSongRoom;
  // Standing in for the list, the note keeps the room a row would, rather
  // than sitting tight under the search.
  if (shown.length === 0)
    return ofKind.length === 0 ? (
      <Alert className="py-inset">
        {kind === "music"
          ? t("editor-island-sound-library-empty-music")
          : t("editor-island-sound-library-empty-effects")}
      </Alert>
    ) : (
      <Alert className="py-inset" icon={<SearchX />}>
        {t("editor-island-sound-library-no-match")}
      </Alert>
    );
  return (
    <ul aria-label={t("editor-island-sound-library")} className="flex flex-col">
      {shown.map((sound) => (
        <LibraryRow
          isPlaying={preview.isPlaying(sound)}
          key={sound.name}
          onAdd={() => {
            if (!isFull) onAdd({ kind, name: sound.name });
          }}
          onDrag={() => {
            const look = soundDragLook(sound);
            if (!look) return;
            dragLibrarySound(sound.name, look).catch((cause: unknown) => {
              console.error("Could not drag the sound", cause);
            });
          }}
          onPlayFrom={(fromMs) => {
            preview.playFrom(sound, fromMs);
          }}
          onRename={(title) => {
            renameSound(sound.name, title).catch((cause: unknown) => {
              console.error("Could not rename the sound", cause);
            });
          }}
          onTogglePlay={() => {
            preview.toggle(sound);
          }}
          played={preview.playedOf(sound)}
          sound={sound}
        />
      ))}
    </ul>
  );
}
