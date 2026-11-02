// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ISLAND } from "../../../primitives/island/island-geometry";
import {
  previewCamera,
  previewMicrophone,
  previewSystemAudio,
} from "../../recording-controls/island/island-preview";
import { useStageCamera } from "../../recording-controls/island/island-stage-state";
import { RecordingIsland } from "../../recording-controls/island/recording-island";
import { useDisplayNotches } from "../preview/use-display-notches";

import { StageDesktop } from "./stage-desktop";
import { StageMenuBar } from "./stage-menu-bar";
import { StageScreen } from "./stage-screen";
import { useClock } from "./use-clock";
import { useWallpaperSource } from "./use-wallpaper-source";

import type {
  IslandMoment,
  IslandPhase,
} from "../../recording-controls/island/island-types";

// The real recording island, played on a loop over the person's desktop,
// so what a setting does is shown by the island itself rather than a
// drawing of it. It comes out as it would on this Mac: from under the
// menu bar's notch on a display with one, hanging just below the top of
// one without. Each scene is a timeline read from one clock: the island
// coming out as the app starts, the countdown and the start it leads to, a
// recording with its time running, or moments placed one after another.
// Nothing on it can be pressed; the pane's rows change it. The screen is
// scaled down where the stage cannot hold the island whole, measured as it
// is drawn: under a notch the island is the notch's width wider, and the
// row either side of it grows with the settings (`StageScreen`).

export type IslandScene =
  | { kind: "countdown"; seconds: number }
  | {
      kind: "moments";
      kinds: { color: string; name: string }[];
      notes: boolean;
    }
  | { kind: "recording" }
  | { kind: "startup"; showsControls: boolean };

/** A moment placed from outside the loop, such as by pressing its shortcut
 * while the pane is open. */
export type PlacedMoment = { color: string; key: number; name: string };

const STARTING = 1.2;
const RECORDED = 3;
const MOMENT_CYCLE = 3.2;
/** Startup: how long the desktop shows before the island comes out, when
 * it tucks away again, and one pass, which leaves the desktop bare for a
 * moment after the island has gone back in. */
const LAUNCHED = 0.8;
const TUCKED = 4.2;
const STARTUP = 5.2;
const noop = () => undefined;
const findIsland = (screen: HTMLElement) =>
  screen.querySelector("[data-island-outline]");

/** The least room the island takes on its screen, with its shadow: its
 * row, or its row and the panel it comes out with as the app starts. */
const ROW_ONLY = { height: 120, width: 520 };
const WITH_PANEL = { height: 270, width: 600 };

type Frame = {
  countdown: number;
  elapsed: number;
  isShown: boolean;
  moment: IslandMoment | null;
  phase: IslandPhase;
};

/** Where a scene's timeline is at a time. */
function frameAt(scene: IslandScene, time: number): Frame {
  const none = { countdown: 3, isShown: true, moment: null };
  if (scene.kind === "startup") {
    const at = time % STARTUP;
    return {
      ...none,
      elapsed: 0,
      isShown: scene.showsControls && at >= LAUNCHED && at < TUCKED,
      phase: "idle",
    };
  }
  if (scene.kind === "countdown") {
    const lead = scene.seconds + STARTING;
    const at = time % (lead + RECORDED);
    if (at < scene.seconds)
      return {
        ...none,
        countdown: scene.seconds - Math.floor(at),
        elapsed: 0,
        phase: "countdown",
      };
    if (at < lead) return { ...none, elapsed: 0, phase: "starting" };
    return { ...none, elapsed: at - lead, phase: "recording" };
  }
  const elapsed = 42 + time;
  if (scene.kind === "recording" || scene.kinds.length === 0)
    return { ...none, elapsed, phase: "recording" };
  // One moment per cycle, each kind in turn; with voice notes on, every
  // other one is held long enough to take a note.
  const index = Math.floor(time / MOMENT_CYCLE);
  const at = time % MOMENT_CYCLE;
  const kind = scene.kinds[index % scene.kinds.length];
  const isNote = scene.notes && index % 2 === 1;
  const shownFor = isNote ? 2.6 : 1.6;
  const moment: IslandMoment | null =
    at < shownFor
      ? {
          color: kind.color,
          key: index,
          name: kind.name,
          noMicrophone: false,
          noteMs: isNote && at > 0.5 ? (at - 0.5) * 1000 : null,
        }
      : null;
  return { ...none, elapsed, moment, phase: "recording" };
}

const sceneKey = (scene: IslandScene) =>
  scene.kind === "countdown"
    ? `countdown ${String(scene.seconds)}`
    : scene.kind === "startup"
      ? `startup ${String(scene.showsControls)}`
      : scene.kind;

export function LiveIsland({
  hasConfidenceChecks = false,
  isOn = true,
  placed,
  scene,
  showsSize = false,
}: {
  scene: IslandScene;
  hasConfidenceChecks?: boolean;
  /** The setting the picture shows is on; off, the desktop is drained of
   * colour (`StageDesktop`). */
  isOn?: boolean;
  placed?: PlacedMoment | null;
  showsSize?: boolean;
}) {
  const time = useClock(sceneKey(scene));
  const frame = frameAt(scene, time);
  const camera = useStageCamera(false, false, true);
  const { island: notch } = useDisplayNotches();
  const wallpaper = useWallpaperSource();
  // Someone talking: the microphone moves quickly, the computer's sound
  // more slowly under it.
  const microphoneLevel =
    0.25 + 0.55 * Math.abs(Math.sin(time * 5.3) * Math.sin(time * 1.7 + 1));
  const systemAudioLevel = 0.2 + 0.3 * Math.abs(Math.sin(time * 2.1));
  const moment = placed
    ? { ...placed, key: 1000 + placed.key, noMicrophone: false, noteMs: null }
    : frame.moment;

  return (
    <StageScreen
      fits={scene.kind === "startup" ? WITH_PANEL : ROW_ONLY}
      subject={{
        find: findIsland,
        key: [
          sceneKey(scene),
          hasConfidenceChecks,
          showsSize,
          notch?.width ?? "no notch",
        ].join(" "),
      }}
    >
      {(screen) => (
        <StageDesktop isOn={isOn}>
          <StageMenuBar notch={notch} width={screen.width} />
          <div
            className="absolute inset-x-0"
            style={{ top: notch ? 0 : ISLAND.screenGap }}
          >
            <RecordingIsland
              {...camera}
              camera={previewCamera()}
              countdown={frame.countdown}
              displays={[{ id: 1, label: "", thumbnail: wallpaper }]}
              elapsed={frame.elapsed}
              hasConfidenceChecks={hasConfidenceChecks}
              isShown={frame.isShown}
              microphone={previewMicrophone({ level: microphoneLevel })}
              mode="screen"
              moment={moment}
              notch={notch}
              onDeviceChange={noop}
              onInputChange={noop}
              onModeChange={noop}
              onRecord={noop}
              phase={frame.phase}
              selectedDisplayId={1}
              systemAudio={previewSystemAudio({ level: systemAudioLevel })}
              writtenBytes={
                showsSize
                  ? frame.elapsed < 1
                    ? null
                    : Math.round(frame.elapsed * 2_600_000)
                  : undefined
              }
            />
          </div>
        </StageDesktop>
      )}
    </StageScreen>
  );
}
