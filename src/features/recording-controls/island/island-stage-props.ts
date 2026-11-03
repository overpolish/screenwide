// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type {
  IslandDisplay,
  IslandFailure,
  IslandInput,
  IslandMode,
  IslandMoment,
  IslandReplay,
  IslandPanel,
  IslandPhase,
  IslandRegion,
  IslandWindow,
  RecordingIslandProps,
} from "./island-types";

// What a story can set on the island's stage: the island's own props the
// stage passes through, and the settings and states it starts from.

export type IslandStageProps = Pick<
  RecordingIslandProps,
  | "annotate"
  | "notch"
  | "isScreenLocked"
  | "isShown"
  | "onCameraFlipChange"
  | "onCancelCountdown"
  | "onCancelStarting"
  | "onCancelWindowPick"
  | "onChooseWindow"
  | "onDiscard"
  | "onDismiss"
  | "onDismissFailure"
  | "onDisplayChange"
  | "onHidden"
  | "onOpenPermissions"
  | "onPanelChange"
  | "onPause"
  | "onReplayChange"
  | "onResume"
  | "onScreenshot"
  | "onShapeChange"
  | "onStop"
  | "windowPick"
> &
  // The island needs these; the stage answers them itself, and tells the
  // story too, so its Actions log every event.
  Partial<
    Pick<
      RecordingIslandProps,
      "onDeviceChange" | "onInputChange" | "onModeChange" | "onRecord"
    >
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
