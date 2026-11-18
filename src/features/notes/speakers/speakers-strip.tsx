// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useNumberFormatter } from "react-aria";

import { EditableTitle } from "../../../composites/editable-title/editable-title";
import { t } from "../../../i18n/i18n";
import { IconToggleButton } from "../../../primitives/button/icon-button";
import { Text } from "../../../primitives/text/text";

import { voiceNumber } from "./use-voice-clips";
import { VoiceAvatar } from "./voice-avatar";

// Who was on the call, above the notes: each speaker as a pill of their
// avatar, their name and how much of the talking was theirs, in the order
// they first spoke. A name is edited where it stands and renames them
// throughout, as the name over their words does. A speaker not yet named
// can be heard, to tell who they are, until they are: their avatar plays
// their voice, and clicking their name selects it whole to type over. Their
// share is a toggle that shows only what the chosen speakers said.

export function SpeakersStrip({
  chosen,
  isReadOnly,
  onChoose,
  onHearVoice,
  onRename,
  playingVoice,
  speakers,
  voices,
}: {
  /** The speakers whose words alone are shown; none shows everyone's. */
  chosen: readonly string[];
  isReadOnly: boolean;
  /** Chooses `name`, or lets them go if they were chosen. */
  onChoose: (name: string) => void;
  onHearVoice: (number: number) => void;
  onRename: (from: string, to: string) => void;
  /** The speaker whose voice is playing, by number; null when none is. */
  playingVoice: number | null;
  /** Each speaker's share of the words said, from 0 to 1. */
  speakers: readonly { name: string; share: number }[];
  /** The numbers of the speakers whose voice can be heard. */
  voices: readonly number[];
}) {
  const percent = useNumberFormatter({
    maximumFractionDigits: 0,
    style: "percent",
  });
  return (
    <div className="flex flex-col gap-inline">
      <ul
        aria-label={t("notes-view-speakers")}
        className="flex flex-wrap gap-inline"
      >
        {speakers.map(({ name, share }, index) => {
          const voice = voiceNumber(name);
          const canHear = voice !== null && voices.includes(voice);
          return (
            // Keyed by place, which a rename keeps, so the pill renamed is
            // the same pill and its avatar can change over where it stands.
            <li
              className="flex items-center gap-inline rounded-full bg-fill p-1"
              // eslint-disable-next-line @eslint-react/no-array-index-key
              key={index}
            >
              <VoiceAvatar
                isPlaying={canHear && playingVoice === voice}
                name={name}
                onHear={
                  canHear
                    ? () => {
                        onHearVoice(voice);
                      }
                    : undefined
                }
              />
              {isReadOnly ? (
                <span className="text-label text-fg" dir="auto">
                  {name}
                </span>
              ) : (
                <EditableTitle
                  className="text-label text-fg"
                  label={t("notes-view-speaker-name", { speaker: name })}
                  onChange={(to) => {
                    onRename(name, to);
                  }}
                  selectsAllOnFocus={voice !== null}
                  title={name}
                />
              )}
              {/* The share as a pill the avatar's height at the pill's
                  other end, padded as the island pads its pills, which is
                  tighter than a button. */}
              <IconToggleButton
                aria-label={t("notes-view-speaker-filter", {
                  share: percent.format(share),
                  speaker: name,
                })}
                className="h-7 rounded-full px-inset-sm tabular-nums"
                isSelected={chosen.includes(name)}
                label={percent.format(share)}
                onChange={() => {
                  onChoose(name);
                }}
              >
                {null}
              </IconToggleButton>
            </li>
          );
        })}
      </ul>
      {voices.length > 0 ? (
        <Text variant="caption">{t("notes-view-voices-kept")}</Text>
      ) : null}
    </div>
  );
}
