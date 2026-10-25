// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  PREVIEW_DISPLAYS,
  previewCamera,
  previewMicrophone,
  previewSystemAudio,
} from "../../../../features/recording-controls/island/island-preview";
import { useStageCamera } from "../../../../features/recording-controls/island/island-stage-state";
import { RecordingIsland } from "../../../../features/recording-controls/island/recording-island";
import { ISLAND } from "../../../../primitives/island/island-geometry";

import { useClock } from "./use-clock";

import type {
  IslandMoment,
  IslandPhase,
} from "../../../../features/recording-controls/island/island-types";

// The real recording island, played on a loop for a settings pane, so what a
// setting does is shown by the island itself rather than a drawing of it.
// Each scene is a timeline read from one clock: the countdown and the start
// it leads to, a recording with its time running, or moments placed one
// after another. Nothing on it can be pressed; the pane's rows change it.

export type IslandScene =
  | { kind: "countdown"; seconds: number }
  | {
      kind: "moments";
      kinds: { color: string; name: string }[];
      notes: boolean;
    }
  | { kind: "recording" };

/** A moment placed from outside the loop, such as by pressing its shortcut
 * while the pane is open. */
export type FlashedMoment = { color: string; key: number; name: string };

const STARTING = 1.2;
const RECORDED = 3;
const MOMENT_CYCLE = 3.2;
const noop = () => undefined;

type Frame = {
  countdown: number;
  elapsed: number;
  moment: IslandMoment | null;
  phase: IslandPhase;
};

/** Where a scene's timeline is at a time. */
function frameAt(scene: IslandScene, time: number): Frame {
  const none = { countdown: 3, moment: null };
  if (scene.kind === "countdown") {
    const lead = scene.seconds + STARTING;
    const at = time % (lead + RECORDED);
    if (at < scene.seconds)
      return {
        countdown: scene.seconds - Math.floor(at),
        elapsed: 0,
        moment: null,
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
  return { countdown: 3, elapsed, moment, phase: "recording" };
}

export function LiveIsland({
  flash,
  hasConfidenceChecks,
  scene,
  showsSize,
}: {
  hasConfidenceChecks: boolean;
  scene: IslandScene;
  showsSize: boolean;
  flash?: FlashedMoment | null;
}) {
  const time = useClock(
    scene.kind === "countdown"
      ? `countdown ${String(scene.seconds)}`
      : scene.kind,
  );
  const frame = frameAt(scene, time);
  const camera = useStageCamera(false, false, true);
  // Someone talking: the microphone moves quickly, the computer's sound
  // more slowly under it.
  const microphoneLevel =
    0.25 + 0.55 * Math.abs(Math.sin(time * 5.3) * Math.sin(time * 1.7 + 1));
  const systemAudioLevel = 0.2 + 0.3 * Math.abs(Math.sin(time * 2.1));
  const moment = flash
    ? { ...flash, key: 1000 + flash.key, noMicrophone: false, noteMs: null }
    : frame.moment;

  return (
    <div className="absolute inset-x-0" style={{ top: ISLAND.screenGap }}>
      <RecordingIsland
        {...camera}
        camera={previewCamera()}
        countdown={frame.countdown}
        displays={PREVIEW_DISPLAYS}
        elapsed={frame.elapsed}
        hasConfidenceChecks={hasConfidenceChecks}
        microphone={previewMicrophone({ level: microphoneLevel })}
        mode="screen"
        moment={moment}
        onDeviceChange={noop}
        onInputChange={noop}
        onModeChange={noop}
        onRecord={noop}
        phase={frame.phase}
        selectedDisplayId={PREVIEW_DISPLAYS[0].id}
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
  );
}
