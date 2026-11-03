// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";
import { useLocale } from "react-aria";

import { ISLAND } from "../../../primitives/island/island-geometry";

import {
  PREVIEW_DISPLAYS,
  PREVIEW_WINDOWS,
  previewCamera,
  previewMicrophone,
  previewSystemAudio,
} from "./island-fixtures";
import {
  stageMicrophone,
  stageWrittenBytes,
  useStageAppearing,
  useStageCamera,
  useStageFailures,
  useStageReplay,
} from "./island-stage-state";
import { RecordingIsland } from "./recording-island";

import type {
  IslandDisplay,
  IslandFailure,
  IslandInput,
  IslandInputKind,
  IslandMode,
  IslandMoment,
  IslandReplay,
  IslandPanel,
  IslandPhase,
  IslandRegion,
  IslandWindow,
  RecordingIslandProps,
} from "./island-types";

// The island's stories: the island at the top of the canvas, under the
// notch when there is one. It plays the recording's part, counting down and
// running the clock, so every story can be used, not only looked at. The
// inputs' levels are set from the story's controls, to try the meters at
// any level.

export type IslandStageProps = Pick<
  RecordingIslandProps,
  | "annotate"
  | "notch"
  | "isScreenLocked"
  | "isShown"
  | "onCancelWindowPick"
  | "onChooseWindow"
  | "onOpenPermissions"
  | "onDismiss"
  | "onScreenshot"
  | "onHidden"
  | "onShapeChange"
  | "windowPick"
> & {
  /** Shows the island coming out and going away, over and over. */
  appears?: boolean;
  camera?: Partial<IslandInput>;
  /** The area already dragged out. */
  chosenRegion?: IslandRegion;
  // Choose a Window picks the sample windows in turn, standing in for the
  // picker on the screen.
  /** The window already picked. */
  chosenWindow?: IslandWindow;
  /** The displays to choose from; two when not given. */
  displays?: IslandDisplay[];
  /** Failures Rust would keep, said in turn as OK puts each away. */
  failures?: IslandFailure[];
  /** The confidence checks setting: shows what is recorded from. */
  hasConfidenceChecks?: boolean;
  /** The countdown setting: Record counts down before starting. */
  hasCountdown?: boolean;
  /** Seconds already recorded when the story opens recording. */
  initialElapsed?: number;
  initialMode?: IslandMode;
  initialPanel?: IslandPanel;
  initialPhase?: IslandPhase;
  /** The camera mirrored; Flip on the camera's picture changes it. */
  isCameraFlipped?: boolean;
  /** The camera started but not yet sending its picture. */
  isCameraLoading?: boolean;
  microphone?: Partial<IslandInput>;
  /** Microphone level, 0 to 1. */
  microphoneLevel?: number;
  /** Something wrong with the microphone, named; the stage words it. */
  microphoneProblem?: "missing";
  /** A moment placed by its shortcut; a voice note runs on while recording. */
  moment?: IslandMoment;
  // Turned on, it starts for a moment before it is on.
  /** The replay buffer, off when absent; null where it cannot run. */
  replay?: IslandReplay | null;
  /** The recording size setting: shows the size so far. */
  showsSize?: boolean;
  systemAudio?: Partial<IslandInput>;
  /** System audio level, 0 to 1. */
  systemAudioLevel?: number;
};

export function IslandStage(props: IslandStageProps) {
  const [mode, setMode] = useState<IslandMode>(props.initialMode ?? "screen");
  const [phase, setPhase] = useState<IslandPhase>(props.initialPhase ?? "idle");
  const [display, setDisplay] = useState(1);
  const [chosenWindow, setChosenWindow] = useState(props.chosenWindow);
  const [moment, setMoment] = useState(props.moment ?? null);
  const failures = useStageFailures(props.failures);
  const camera = useStageCamera(
    props.isCameraFlipped,
    props.isCameraLoading,
    phase === "recording" || phase === "paused",
  );
  const [count, setCount] = useState(3);
  // A story that opens on Starting holds it there to be looked at; pressing
  // Record plays it through as the recording would.
  const [holdsStart, setHoldsStart] = useState(
    props.initialPhase === "starting",
  );
  const [elapsed, setElapsed] = useState(
    props.initialPhase === "recording" || props.initialPhase === "paused"
      ? (props.initialElapsed ?? 42)
      : 0,
  );
  const { changeReplay, replay } = useStageReplay(props.replay);

  const [inputs, setInputs] = useState<Record<IslandInputKind, IslandInput>>(
    () => ({
      camera: previewCamera(props.camera),
      microphone: previewMicrophone(props.microphone),
      systemAudio: previewSystemAudio(props.systemAudio),
    }),
  );

  useEffect(() => {
    if (phase !== "recording") return;
    let frame = 0;
    let last = performance.now();
    const tick = (now: number) => {
      const seconds = (now - last) / 1000;
      last = now;
      setElapsed((value) => value + seconds);
      setMoment((current) =>
        current?.noteMs == null
          ? current
          : { ...current, noteMs: current.noteMs + seconds * 1_000 },
      );
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [phase]);

  useEffect(() => {
    if (phase === "countdown") {
      const timer = setTimeout(() => {
        if (count <= 1) setPhase("starting");
        else setCount(count - 1);
      }, 1000);
      return () => {
        clearTimeout(timer);
      };
    }
    // The capture always takes a moment to get going, countdown or not.
    if (phase === "starting" && !holdsStart) {
      const timer = setTimeout(() => {
        setPhase("recording");
      }, 1400);
      return () => {
        clearTimeout(timer);
      };
    }
    if (phase === "finishing") {
      const timer = setTimeout(() => {
        setPhase("idle");
      }, 1600);
      return () => {
        clearTimeout(timer);
      };
    }
  }, [phase, count, holdsStart]);

  const isShown = useStageAppearing(props.appears);
  const { direction } = useLocale();

  const { level, warning: microphoneWarning } = stageMicrophone({
    inputs,
    problem: props.microphoneProblem,
  });

  // Cancelling a start, or discarding a recording, comes back to rest.
  const rest = () => {
    setPhase("idle");
  };

  // The stage is the screen, which stays left to right; the island on it
  // follows the language, as it does in its own window.
  return (
    <div className="relative size-full" dir="ltr">
      <div
        className="absolute inset-x-0"
        dir={direction}
        style={{ top: props.notch ? 0 : ISLAND.screenGap }}
      >
        <RecordingIsland
          {...camera}
          {...failures}
          annotate={props.annotate}
          camera={inputs.camera}
          chosenRegion={props.chosenRegion}
          chosenWindow={chosenWindow}
          countdown={count}
          displays={props.displays ?? PREVIEW_DISPLAYS}
          elapsed={elapsed}
          hasConfidenceChecks={props.hasConfidenceChecks}
          initialPanel={props.initialPanel}
          isScreenLocked={props.isScreenLocked}
          isShown={props.isShown ?? isShown}
          microphone={{
            ...inputs.microphone,
            level: level("microphone", props.microphoneLevel),
            warning: microphoneWarning,
          }}
          mode={mode}
          moment={moment}
          notch={props.notch}
          onCancelCountdown={rest}
          onCancelStarting={rest}
          onCancelWindowPick={props.onCancelWindowPick}
          onChooseWindow={() => {
            props.onChooseWindow?.();
            setChosenWindow((current) =>
              current === PREVIEW_WINDOWS[0]
                ? PREVIEW_WINDOWS[1]
                : PREVIEW_WINDOWS[0],
            );
          }}
          onDeviceChange={(kind, ids) => {
            setInputs((current) => ({
              ...current,
              [kind]: { ...current[kind], selectedIds: ids },
            }));
          }}
          onDiscard={rest}
          onDismiss={props.onDismiss}
          onDisplayChange={setDisplay}
          onHidden={props.onHidden}
          onInputChange={(kind, isOn) => {
            setInputs((current) => ({
              ...current,
              [kind]: { ...current[kind], isOn },
            }));
          }}
          onModeChange={setMode}
          onOpenPermissions={props.onOpenPermissions}
          onPause={() => {
            setPhase("paused");
          }}
          onRecord={() => {
            setCount(3);
            setElapsed(0);
            setHoldsStart(false);
            setPhase(props.hasCountdown === false ? "starting" : "countdown");
          }}
          onReplayChange={changeReplay}
          onResume={() => {
            setPhase("recording");
          }}
          onScreenshot={props.onScreenshot}
          onShapeChange={props.onShapeChange}
          onStop={() => {
            setPhase("finishing");
          }}
          phase={phase}
          replay={replay}
          selectedDisplayId={display}
          systemAudio={{
            ...inputs.systemAudio,
            level: level("systemAudio", props.systemAudioLevel),
          }}
          windowPick={props.windowPick}
          writtenBytes={stageWrittenBytes(props.showsSize, elapsed)}
        />
      </div>
    </div>
  );
}
