// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ReactNode } from "react";

import type {
  IslandAnchor,
  IslandNotch,
} from "../../../primitives/island/island-geometry";
import type { IslandPart } from "../../../primitives/island/island-outline-sync";
import type { ScreenshotAction, ScreenshotState } from "../types";

// What the island shows, handed to it whole: it keeps only which part is
// open. Whoever drives it maps the recording, input and source stores onto
// these, so the stories can drive every state without them.

export type IslandInputKind = "camera" | "microphone" | "systemAudio";

type IslandDevice = {
  id: string;
  label: string;
  /** A camera's picture, width over height; 16:9 when not given. */
  aspectRatio?: number;
  /** An app's icon, for a system audio source that is one app. */
  icon?: string;
  /** Listed first, ahead of the devices sorted by name, such as system
   * audio's whole mix. */
  isPinned?: boolean;
};

export type IslandInput = {
  devices: IslandDevice[];
  isOn: boolean;
  /** The devices in use, by id: at most one for the camera and microphone,
   * none when there is none to use; for system audio, the whole mix alone
   * or any number of apps. */
  selectedIds: string[];
  isLocked?: boolean;
  /** 0 to 1, for the microphone and system audio. */
  level?: number;
  /** Why it cannot be recorded, when it cannot. */
  warning?: string;
};

export type IslandDisplay = { id: number; label: string; thumbnail?: string };

/** A window picked to be recorded: its title, the app it belongs to and the
 * app's icon. */
export type IslandWindow = { appName: string; title: string; icon?: string };

/** An area dragged out to be recorded: its size in pixels and the names of
 * the displays it lies on, which may be several. */
export type IslandRegion = {
  displays: string[];
  height: number;
  width: number;
};

/** A moment placed while recording, by its shortcut: its kind's name and
 * colour, and, held long enough, the voice note it is taking. */
export type IslandMoment = {
  color: string;
  /** Tells two moments of one kind in a row apart. */
  key: number;
  name: string;
  /** Held long enough for a note, with no microphone to take it. */
  noMicrophone: boolean;
  /** How long the note has run; null for a moment alone. */
  noteMs: number | null;
};

/** The replay buffer, where it can run: off, on, or still starting. */
export type IslandReplay = { status: "off" | "on" | "starting" };

/** What is recorded: a display, a window, a region, or no screen at all,
 * only the inputs that are on. The island's own modes; whoever drives it
 * maps no screen onto the recorder's camera or audio mode. */
export type IslandMode = "none" | "region" | "screen" | "window";

/** What the island has grown to show. */
export type IslandPanel = IslandInputKind | "record" | "screenshot" | "source";

export type IslandPhase =
  "countdown" | "finishing" | "idle" | "paused" | "recording" | "starting";

export type RecordingIslandProps = {
  camera: IslandInput;
  displays: IslandDisplay[];
  microphone: IslandInput;
  mode: IslandMode;
  onDeviceChange: (input: IslandInputKind, ids: string[]) => void;
  onInputChange: (input: IslandInputKind, isOn: boolean) => void;
  onModeChange: (mode: IslandMode) => void;
  onRecord: () => void;
  phase: IslandPhase;
  selectedDisplayId: number;
  systemAudio: IslandInput;
  /** The annotate tools, while annotating as it records: a line laid under
   * the recording's row in its black, which stays one row, and what hangs
   * from the line where it is anchored. Whoever drives the island lays them
   * out (`useAnnotateRows`); shown only while recording or paused. */
  annotate?: { anchor: IslandAnchor; options: ReactNode; tools: ReactNode };
  /** The live camera, drawn by whoever drives the island. */
  cameraPreview?: ReactNode;
  /** The area dragged out in the Region mode; none until one is. */
  chosenRegion?: IslandRegion;
  /** The window picked in the Window mode; none until one is. */
  chosenWindow?: IslandWindow;
  /** Seconds left before recording starts. */
  countdown?: number;
  /** Seconds recorded so far. */
  elapsed?: number;
  /** While recording, shows what is recorded from: the confidence checks
   * setting. */
  hasConfidenceChecks?: boolean;
  /** The part it opens on each time it comes out; what will be recorded
   * unless a story starts elsewhere. */
  initialPanel?: IslandPanel;
  /** The camera mirrored, as it is previewed and recorded. */
  isCameraFlipped?: boolean;
  isScreenLocked?: boolean;
  /** Whether it is out; hidden, it is tucked into the notch, or gone. */
  isShown?: boolean;
  /** A moment just placed or still being held, shown in the time's place. */
  moment?: IslandMoment | null;
  /** The notch of the screen the island is on, measured; none where the
   * screen has none. */
  notch?: IslandNotch | null;
  onCameraFlipChange?: (isFlipped: boolean) => void;
  onCancelCountdown?: () => void;
  onCancelStarting?: () => void;
  onCancelWindowPick?: () => void;
  onChooseWindow?: () => void;
  onDiscard?: () => void;
  onDisplayChange?: (id: number) => void;
  /** The island has tucked itself away after `isShown` turned false. */
  onHidden?: () => void;
  onOpenPermissions?: () => void;
  /** The part now open, as it changes: the camera's, say, which wants the
   * camera's picture. */
  onPanelChange?: (panel: IslandPanel) => void;
  onPause?: () => void;
  onReplayChange?: (isOn: boolean) => void;
  onResume?: () => void;
  onScreenshot?: (action: ScreenshotAction | "delayed" | "save") => void;
  /** Where the island's parts are, as they move (`Island`). */
  onShapeChange?: (parts: IslandPart[]) => void;
  onStop?: () => void;
  /** The replay buffer; none where it cannot run. */
  replay?: IslandReplay;
  screenshotState?: ScreenshotState;
  /** A window being picked by clicking it: the one under the pointer, or
   * null over none. None when not picking. */
  windowPick?: { hovered: IslandWindow | null } | null;
  /** How much of the recording is on disk: null until first measured, and
   * undefined when the size is not shown at all. */
  writtenBytes?: number | null;
};
