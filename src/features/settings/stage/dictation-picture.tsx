// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ISLAND } from "../../../primitives/island/island-geometry";
import { DictationIsland } from "../../dictation/island/dictation-island";
import { useDisplayNotches } from "../preview/use-display-notches";

import { type DictationSubject, dictationFrame } from "./dictation-script";
import { StageDesktop } from "./stage-desktop";
import { StageMenuBar } from "./stage-menu-bar";
import { StageScreen } from "./stage-screen";
import { useClock } from "./use-clock";

import type { DictationCommand } from "../../transcription/types";

// The real dictation island at the top of the person's desktop, playing a
// row's script on a loop (`dictation-script`): words heard and settling, a
// command carried out, a word added. Nothing on it can be pressed.

/** The least room the island takes on its screen, with its shadow. */
const FITS = { height: 200, width: 520 };
const noop = () => undefined;
const findIsland = (screen: HTMLElement) =>
  screen.querySelector("[data-island-outline]");

export function DictationPicture({
  commands,
  subject,
}: {
  /** The spoken commands of the language dictation is in. */
  commands: DictationCommand[];
  subject: DictationSubject;
}) {
  const time = useClock(subject);
  const { island: notch } = useDisplayNotches();
  // Someone talking, as the recording island's picture hears them.
  const level =
    0.25 + 0.55 * Math.abs(Math.sin(time * 5.3) * Math.sin(time * 1.7 + 1));
  return (
    <StageScreen
      fits={FITS}
      subject={{
        find: findIsland,
        key: `${subject} ${String(notch?.width ?? "no notch")}`,
      }}
    >
      {(screen) => (
        <StageDesktop>
          <StageMenuBar notch={notch} width={screen.width} />
          <div
            className="absolute inset-x-0"
            style={{ top: notch ? 0 : ISLAND.screenGap }}
          >
            <DictationIsland
              level={level}
              notch={notch}
              onAction={noop}
              state={dictationFrame(subject, time, commands)}
            />
          </div>
        </StageDesktop>
      )}
    </StageScreen>
  );
}
