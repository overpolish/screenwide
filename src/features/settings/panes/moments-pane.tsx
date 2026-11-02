// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Ref, useEffect, useState } from "react";

import { t } from "../../../i18n/i18n";
import { hotkeyFromEvent, hotkeysEqual } from "../../../lib/hotkey";
import { MomentsSettingsPanel } from "../sections/moments-settings";
import { LiveIsland, type PlacedMoment } from "../stage/live-island";
import { PreviewStage } from "../stage/preview-stage";

import { SettingsPane } from "./settings-pane";

import type { MomentSettingsState } from "../sections/use-moment-settings";
import type { TranscriptionControls } from "../sections/use-transcription";

// Moments: the island placing each kind in turn, in its colour and under
// its name, as they are now, so a rename or a new colour shows at once.
// With voice notes on, every other one is held for a note. Pressing a
// moment's shortcut while the pane is open places that one, to try the
// key: the keys are only the app's while a recording runs, so here they
// reach the page.

const PLACED_MS = 1600;

export function MomentsPane({
  moments,
  onOpenShortcut,
  paneRef,
  transcription,
}: {
  moments: MomentSettingsState;
  onOpenShortcut: (title: string) => void;
  transcription: TranscriptionControls;
  paneRef?: Ref<HTMLElement>;
}) {
  const [placed, setPlaced] = useState<PlacedMoment | null>(null);
  const kinds = moments.settings?.kinds;

  useEffect(() => {
    if (!kinds) return;
    const press = (event: KeyboardEvent) => {
      // A shortcut field taking the key keeps it to itself.
      if (event.defaultPrevented) return;
      const pressed = hotkeyFromEvent(event);
      const kind = kinds.find(
        (candidate) =>
          pressed !== null &&
          candidate.shortcut !== null &&
          hotkeysEqual(candidate.shortcut, pressed),
      );
      if (!kind) return;
      event.preventDefault();
      setPlaced((current) => ({
        color: kind.color,
        key: (current?.key ?? 0) + 1,
        name: kind.name,
      }));
    };
    window.addEventListener("keydown", press);
    return () => {
      window.removeEventListener("keydown", press);
    };
  }, [kinds]);

  useEffect(() => {
    if (!placed) return;
    const timer = window.setTimeout(() => {
      setPlaced(null);
    }, PLACED_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [placed]);

  if (!moments.settings || !kinds) return null;

  return (
    <SettingsPane
      label={t("settings-section-moments")}
      paneRef={paneRef}
      stage={
        <PreviewStage subject="moments">
          <LiveIsland
            placed={placed}
            scene={{
              kind: "moments",
              kinds: kinds.map(({ color, name }) => ({ color, name })),
              notes: moments.settings.voiceNotes,
            }}
          />
        </PreviewStage>
      }
    >
      <MomentsSettingsPanel
        moments={moments}
        onOpenShortcut={onOpenShortcut}
        transcription={transcription}
      />
    </SettingsPane>
  );
}
