// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AudioLines, Music } from "lucide-react";
import { useEffect, useState } from "react";

import { t } from "../../../../i18n/i18n";
import { PillGroup } from "../../../../primitives/pill-group/pill-group";
import { Text } from "../../../../primitives/text/text";
import { AnnotationChoiceRow } from "../annotation/annotation-island-parts";

import type { AudioClipKind } from "../../../../bindings/AudioClipKind";

// Whether the clip in hand is a song or a sound effect. Songs never
// overlap, so a sound effect made a song needs room among them; where it
// has none, pressing Song changes nothing and says so beside the choice, at
// the row's end, for a few seconds, then the note goes again.

/** How long the note that the songs have no room shows, in ms. */
const NO_ROOM_NOTE_MS = 3_000;

export function ClipKindRow({
  hasSongRoom,
  kind,
  onChange,
}: {
  /** Whether the clip, as a song, would have room among the songs. */
  hasSongRoom: boolean;
  kind: AudioClipKind;
  onChange: (kind: AudioClipKind) => void;
}) {
  const [isNoted, setIsNoted] = useState(false);
  useEffect(() => {
    if (!isNoted) return;
    const timer = window.setTimeout(() => {
      setIsNoted(false);
    }, NO_ROOM_NOTE_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [isNoted]);
  return (
    <AnnotationChoiceRow title={t("editor-island-audio-kind")}>
      {isNoted ? (
        <Text className="text-fg-secondary" key="no-room" variant="caption">
          {t("editor-island-audio-no-song-room")}
        </Text>
      ) : null}
      <PillGroup
        aria-label={t("editor-island-audio-kind")}
        items={[
          {
            ariaLabel: t("editor-island-audio-song"),
            icon: <Music aria-hidden />,
            id: "music",
          },
          {
            ariaLabel: t("editor-island-audio-effect"),
            icon: <AudioLines aria-hidden />,
            id: "effect",
          },
        ]}
        key="kind"
        onSelectionChange={(next) => {
          if (next === "music" && kind !== "music" && !hasSongRoom) {
            setIsNoted(true);
            return;
          }
          onChange(next as AudioClipKind);
        }}
        selected={kind}
      />
    </AnnotationChoiceRow>
  );
}
