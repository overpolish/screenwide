// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import { MomentsSettingsPanel } from "../../../../features/settings/sections/moments-settings";
import { useSettingsApi } from "../../../../features/settings/settings-api-context";
import { useShortcutCapture } from "../../../../features/settings/use-shortcut-capture";
import { hotkeyFromEvent, hotkeysEqual } from "../../../../lib/hotkey";
import { type FlashedMoment, LiveIsland } from "../preview/live-island";
import { PreviewStage, ScreenTop } from "../preview/preview-stage";
import { comboText, parseCombo } from "../shortcuts/shortcut-model";

import { Pane } from "./pane";

import type { ExploredSettings } from "../settings-state";

// Moments: the island placing each kind in turn, in its colour and under
// its name, as they are now, so a rename or a new colour shows at once.
// With voice notes on, every other one is held for a note. Pressing a
// moment's shortcut while the pane is open places that one, to try the key.

const FLASH_MS = 1600;

export function MomentsPane({ settings }: { settings: ExploredSettings }) {
  const { moments, transcription } = settings;
  const api = useSettingsApi();
  const onCaptureChange = useShortcutCapture(
    api.beginShortcutCapture,
    api.endShortcutCapture,
  );
  const [flash, setFlash] = useState<FlashedMoment | null>(null);
  const kinds = moments.settings?.kinds;

  useEffect(() => {
    if (!kinds) return;
    const press = (event: KeyboardEvent) => {
      const pressed = hotkeyFromEvent(event);
      const kind = kinds.find(
        (candidate) =>
          pressed !== null &&
          candidate.shortcut !== null &&
          hotkeysEqual(candidate.shortcut, pressed),
      );
      if (!kind) return;
      event.preventDefault();
      setFlash((current) => ({
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
    if (!flash) return;
    const timer = window.setTimeout(() => {
      setFlash(null);
    }, FLASH_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [flash]);

  if (!moments.settings || !kinds) return null;
  const first = kinds.find((kind) => kind.shortcut !== null);
  const combo = parseCombo(first?.shortcut ?? null);
  const tryIt =
    first && combo
      ? `Press ${comboText(combo)} to place ${first.name}`
      : "Give a moment a shortcut to place it";

  return (
    <Pane
      label="Moments"
      stage={
        <PreviewStage
          sceneKey="moments"
          title={flash ? `${flash.name}, placed` : "While recording"}
          value={tryIt}
        >
          <ScreenTop>
            <LiveIsland
              flash={flash}
              hasConfidenceChecks={false}
              scene={{
                kind: "moments",
                kinds: kinds.map(({ color, name }) => ({ color, name })),
                notes: moments.settings.voiceNotes,
              }}
              showsSize={false}
            />
          </ScreenTop>
        </PreviewStage>
      }
    >
      <MomentsSettingsPanel
        moments={moments}
        onCaptureChange={onCaptureChange}
        transcription={transcription}
      />
    </Pane>
  );
}
