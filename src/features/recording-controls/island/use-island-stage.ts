// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

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

import type { IslandStageProps } from "./island-stage-props";
import type {
  IslandInput,
  IslandInputKind,
  IslandMode,
  IslandPhase,
  RecordingIslandProps,
} from "./island-types";

// The island's stories: the island at the top of the canvas, under the
// notch when there is one. It plays the recording's part, counting down and
// running the clock, so every story can be used, not only looked at. The
// inputs' levels are set from the story's controls, to try the meters at
// any level. The website plays it the same way, with the island in its page.

/** The island's props as the stage plays them. */
export function useIslandStage(props: IslandStageProps): RecordingIslandProps {
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

  const { level, warning: microphoneWarning } = stageMicrophone({
    inputs,
    problem: props.microphoneProblem,
  });

  // Cancelling a start, or discarding a recording, comes back to rest.
  const rest = (told?: () => void) => () => {
    told?.();
    setPhase("idle");
  };

  return {
    ...camera,
    ...failures,
    annotate: props.annotate,
    camera: inputs.camera,
    chosenRegion: props.chosenRegion,
    chosenWindow,
    countdown: count,
    displays: props.displays ?? PREVIEW_DISPLAYS,
    elapsed,
    hasConfidenceChecks: props.hasConfidenceChecks,
    initialPanel: props.initialPanel,
    isScreenLocked: props.isScreenLocked,
    isShown: props.isShown ?? isShown,
    microphone: {
      ...inputs.microphone,
      level: level("microphone", props.microphoneLevel),
      warning: microphoneWarning,
    },
    mode,
    moment,
    notch: props.notch,
    onCameraFlipChange: (isFlipped) => {
      props.onCameraFlipChange?.(isFlipped);
      camera.onCameraFlipChange(isFlipped);
    },
    onCancelCountdown: rest(props.onCancelCountdown),
    onCancelStarting: rest(props.onCancelStarting),
    onCancelWindowPick: props.onCancelWindowPick,
    onChooseWindow: () => {
      props.onChooseWindow?.();
      setChosenWindow((current) =>
        current === PREVIEW_WINDOWS[0]
          ? PREVIEW_WINDOWS[1]
          : PREVIEW_WINDOWS[0],
      );
    },
    onDeviceChange: (kind, ids) => {
      props.onDeviceChange?.(kind, ids);
      setInputs((current) => ({
        ...current,
        [kind]: { ...current[kind], selectedIds: ids },
      }));
    },
    onDiscard: rest(props.onDiscard),
    onDismiss: props.onDismiss,
    onDismissFailure: (id) => {
      props.onDismissFailure?.(id);
      failures.onDismissFailure(id);
    },
    onDisplayChange: (id) => {
      props.onDisplayChange?.(id);
      setDisplay(id);
    },
    onHidden: props.onHidden,
    onInputChange: (kind, isOn) => {
      props.onInputChange?.(kind, isOn);
      setInputs((current) => ({
        ...current,
        [kind]: { ...current[kind], isOn },
      }));
    },
    onModeChange: (next) => {
      props.onModeChange?.(next);
      setMode(next);
    },
    onOpenPermissions: props.onOpenPermissions,
    onPanelChange: props.onPanelChange,
    onPause: () => {
      props.onPause?.();
      setPhase("paused");
    },
    onRecord: () => {
      props.onRecord?.();
      setCount(3);
      setElapsed(0);
      setHoldsStart(false);
      setPhase(props.hasCountdown === false ? "starting" : "countdown");
    },
    onReplayChange: (isOn) => {
      props.onReplayChange?.(isOn);
      changeReplay(isOn);
    },
    onResume: () => {
      props.onResume?.();
      setPhase("recording");
    },
    onScreenshot: props.onScreenshot,
    onShapeChange: props.onShapeChange,
    onStop: () => {
      props.onStop?.();
      setPhase("finishing");
    },
    phase,
    replay,
    selectedDisplayId: display,
    systemAudio: {
      ...inputs.systemAudio,
      level: level("systemAudio", props.systemAudioLevel),
    },
    windowPick: props.windowPick,
    writtenBytes: stageWrittenBytes(props.showsSize, elapsed),
  };
}
