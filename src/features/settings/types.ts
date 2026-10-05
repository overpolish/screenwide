// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  AnnotationHead,
  AnnotationKind,
} from "../../components/shared/annotation-style/types";
import { BackgroundPreset } from "../../components/shared/background-picker/background";
import { SceneTemplate } from "../editor/recording/scenes/recording-scene-template";

export type ShortcutAction =
  | "toggleRecordingBar"
  | "startStopRecording"
  | "pauseResumeRecording"
  | "takeScreenshot"
  | "takeScreenshotToClipboard"
  | "recognizeText"
  | "annotateOverlay"
  | "annotateClear"
  | "rulerOverlay";

type ShortcutBinding = {
  action: ShortcutAction;
  shortcut: string | null;
};

export type ShortcutSettings = {
  bindings: ShortcutBinding[];
};

export type RulerAction =
  | "toggleCrosshair"
  | "copyColour"
  | "deleteMeasurement"
  | "copyMeasurement"
  | "undo"
  | "redo"
  | "stampHorizontal"
  | "stampVertical"
  | "guideVertical"
  | "guideHorizontal"
  | "cycleTolerance"
  | "measureRadius"
  | "toggleCenterlines";

export type RulerSettings = {
  bindings: Record<RulerAction, string | null>;
  enabled: boolean;
};

export type OcrAction = "selectAll" | "copyText";

export type OcrSettings = {
  bindings: Record<OcrAction, string | null>;
  enabled: boolean;
};

/** Where the toolbar was last dropped: logical points from the top-left of
 * the display it landed on. */
type AnnotateToolbarPosition = {
  displayId: number;
  x: number;
  y: number;
};

export type AnnotateSettings = {
  defaultColor: string;
  /** Where a fresh counter's tail points, in radians clockwise from east. A
   * live annotation cannot be picked up again, so the aim is chosen before the
   * counter is dropped. */
  defaultCounterAngle: number;
  /** The disc a fresh counter is drawn at, in output pixels. Kept apart from
   * the arrow's stroke: they are different measurements of different
   * things. */
  defaultCounterSize: number;
  defaultHead: AnnotationHead;
  /** What a fresh stroke is: the editor's own kind. */
  defaultShape: AnnotationKind;
  defaultWidth: number;
  enabled: boolean;
  /** Whether a fresh highlight is drawn as a marker stroke by hand. */
  highlightHandDrawn: boolean;
  /** Whether a fresh highlight is laid by hand over the box its drag spans
   * rather than fitted to the text under it. */
  highlightManual: boolean;
  /** Whether a fresh highlight tints what it covers rather than recolouring
   * the page under it. */
  highlightTint: boolean;
  keepAnnotationsBetweenSessions: boolean;
  /** Whether a fresh shape is drawn as a pen stroke by hand rather than as
   * a clean outline. Its own choice rather than the highlight's. */
  shapeHandDrawn: boolean;
  /** How round a fresh shape's corners are, as a percentage of its shorter
   * side from 0 to 50. */
  shapeRadius: number;
  /** Whether a fresh spotlight also blurs what lies outside it. */
  spotlightBlur: boolean;
  /** How round a fresh spotlight's corners are, as a percentage of its
   * shorter side from 0 to 50. */
  spotlightRadius: number;
  /** How far a fresh spotlight's edge fades, as a percentage of its shorter
   * side from 0 to 50. */
  spotlightSoftness: number;
  toolbarPosition: AnnotateToolbarPosition | null;
};

export type ShortcutDefaults = {
  glide: GlideSettings;
  ocr: OcrSettings;
  ruler: RulerSettings;
  shortcuts: ShortcutSettings;
};

type GlideControl = string;

export type GlideSettings = {
  cursorFollows: boolean;
  doubleTapCenter: boolean;
  enabled: boolean;
  haptics: boolean;
  monitorsModifier: GlideControl;
  mouseModifier: GlideControl;
  spacesModifier: GlideControl;
  thirdsModifier: GlideControl;
  windowGap: number;
};

export type GeneralSettings = {
  accent: "screenwide" | "system";
  /** Colours the annotation tools were given that none of their presets
   * offers, newest last. */
  annotationColors: string[];
  /** Whether a new recording opens with zoom scenes made from its clicks and
   * typing. */
  autoZoom: boolean;
  /** Backgrounds saved from the editor's background picker. */
  backgroundPresets: BackgroundPreset[];
  launchAtLogin: boolean;
  openLocationAfterExport: boolean;
  recordScreenwideWindows: boolean;
  recordingCountdownSeconds: 0 | 3 | 5;
  recordingDirectory: string | null;
  /** Custom scene layouts saved from the Scene panel, oldest first. */
  sceneTemplates: SceneTemplate[];
  screenshotDelaySeconds: 3 | 5 | 10;
  screenshotDirectory: string | null;
  showRecordingBarOnLaunch: boolean;
  showRecordingConfidenceChecks: boolean;
};
