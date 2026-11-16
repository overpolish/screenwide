// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Volume2 } from "lucide-react";
import { useNumberFormatter } from "react-aria";

import { EditableTitle } from "../../../composites/editable-title/editable-title";
import { t } from "../../../i18n/i18n";
import { IconToggleButton } from "../../../primitives/button/icon-button";
import { Text } from "../../../primitives/text/text";

import { SpeakerAvatar } from "./speaker-avatar";
import { voiceNumber } from "./use-voice-clips";

// Who was on the call, above the notes: each speaker as a pill of their
// avatar, their name and how much of the talking was theirs, in the order
// they first spoke. A name is edited where it stands and renames them
// throughout, as the name over their words does. A speaker not yet named
// can be heard, to tell who they are, until they are.

export function SpeakersStrip({
  isReadOnly,
  onHearVoice,
  onRename,
  playingVoice,
  speakers,
  voices,
}: {
  isReadOnly: boolean;
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
        {speakers.map(({ name, share }) => {
          const voice = voiceNumber(name);
          const canHear = voice !== null && voices.includes(voice);
          return (
            <li
              className="flex items-center gap-inline rounded-full bg-fill py-1 ps-1 pe-inset"
              key={name}
            >
              <SpeakerAvatar name={name} />
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
                  title={name}
                />
              )}
              <Text
                aria-label={t("notes-view-speaker-share", {
                  share: percent.format(share),
                })}
                className="tabular-nums"
                variant="caption"
              >
                {percent.format(share)}
              </Text>
              {canHear ? (
                <IconToggleButton
                  aria-label={t("notes-view-speaker-hear", { speaker: name })}
                  className="-me-2 rounded-full"
                  isSelected={playingVoice === voice}
                  onChange={() => {
                    onHearVoice(voice);
                  }}
                >
                  <Volume2 aria-hidden className="size-icon-sm" />
                </IconToggleButton>
              ) : null}
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
