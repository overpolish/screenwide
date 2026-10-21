// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ContextType, useState } from "react";

import { featureStage } from "../../storybook/feature-story-stage";

import { SettingsApiContext } from "./settings-api-context";
import { momentsPreviewApi } from "./settings-moments-preview";
import {
  transcriptionPreviewApi,
  type TranscriptionPreviewSeed,
} from "./settings-transcription-preview";
import { SoftwareUpdateSetting } from "./settings-update-actions";
import { SettingsWindow } from "./settings-window";

import type { AnnotateSettings } from "../../bindings/AnnotateSettings";
import type { GeneralSettings } from "../../bindings/GeneralSettings";
import type { GlideSettings } from "../../bindings/GlideSettings";
import type { OcrSettings } from "../../bindings/OcrSettings";
import type { RulerSettings } from "../../bindings/RulerSettings";
import type { ShortcutDefaults } from "../../bindings/ShortcutDefaults";
import type { ShortcutSettings } from "../../bindings/ShortcutSettings";
import type { Meta } from "@storybook/react-vite";

// Whether a story named where the models start.
const isModelSeed = (value: unknown): value is TranscriptionPreviewSeed =>
  typeof value === "object" &&
  value !== null &&
  "status" in value &&
  "progress" in value;

function createPreviewApi(
  modelSeed?: TranscriptionPreviewSeed,
): ContextType<typeof SettingsApiContext> {
  let general: GeneralSettings = {
    annotationColors: [],
    autoZoom: true,
    backgroundPresets: [],
    captionLayout: {},
    captionSettings: {
      censorProfanity: false,
      hideFillers: false,
      noGaps: false,
      profanity: "mask",
      removePunctuation: false,
    },
    captionStyles: [],
    captionTemplate: "box",
    launchAtLogin: false,
    openLocationAfterExport: true,
    projectDirectory: null,
    recordScreenwideWindows: true,
    recordingCountdownSeconds: 3,
    recordingDirectory: null,
    sceneTemplates: [],
    screenshotDelaySeconds: 3,
    screenshotDirectory: null,
    showRecordingBarOnLaunch: true,
    showRecordingConfidenceChecks: true,
    showRecordingSize: false,
  };
  let glide: GlideSettings = {
    cursorFollows: true,
    doubleTapCenter: true,
    enabled: true,
    haptics: true,
    monitorsModifier: "KeyZ",
    mouseModifier: "AltLeft",
    spacesModifier: "MetaLeft",
    thirdsModifier: "ShiftLeft",
    windowGap: 12,
  };
  let shortcuts: ShortcutSettings = {
    bindings: [
      {
        action: "toggleRecordingBar",
        shortcut: "CommandOrControl+Shift+Digit6",
      },
      { action: "startStopRecording", shortcut: null },
      { action: "pauseResumeRecording", shortcut: null },
      { action: "saveReplay", shortcut: "CommandOrControl+Shift+Digit9" },
      {
        action: "recognizeText",
        shortcut: "CommandOrControl+Shift+KeyT",
      },
      { action: "rulerOverlay", shortcut: "CommandOrControl+Shift+KeyR" },
      { action: "annotateOverlay", shortcut: "CommandOrControl+Shift+KeyA" },
      {
        action: "annotateClear",
        shortcut: "CommandOrControl+Shift+Backspace",
      },
      { action: "takeScreenshot", shortcut: "CommandOrControl+Shift+KeyS" },
    ],
  };
  let ruler: RulerSettings = {
    bindings: {
      copyColour: "Tab",
      copyMeasurement: "CommandOrControl+KeyC",
      cycleTolerance: "KeyT",
      deleteMeasurement: "Backspace",
      guideHorizontal: "KeyH",
      guideVertical: "KeyV",
      measureRadius: "KeyR",
      redo: "CommandOrControl+Shift+KeyZ",
      stampHorizontal: "Digit1",
      stampVertical: "Digit2",
      toggleCenterlines: "KeyM",
      toggleCrosshair: "KeyX",
      undo: "CommandOrControl+KeyZ",
    },
    enabled: true,
  };
  let ocr: OcrSettings = {
    bindings: {
      copyText: "CommandOrControl+KeyC",
      selectAll: "CommandOrControl+KeyA",
    },
    enabled: true,
  };
  let annotate: AnnotateSettings = {
    defaultColor: "#ffcc00",
    defaultCounterAngle: 0,
    defaultCounterSize: 56,
    defaultHead: "end",
    defaultShape: "arrow",
    defaultWidth: 8,
    enabled: true,
    highlightHandDrawn: false,
    highlightManual: false,
    highlightTint: false,
    keepAnnotationsBetweenSessions: false,
    shapeHandDrawn: false,
    shapeRadius: 0,
    spotlightBlur: false,
    spotlightRadius: 12,
    spotlightSoftness: 10,
    toolbarPosition: null,
  };
  const defaults: ShortcutDefaults = {
    glide: { ...glide, windowGap: 0 },
    ocr: { bindings: { ...ocr.bindings }, enabled: ocr.enabled },
    ruler: { bindings: { ...ruler.bindings }, enabled: ruler.enabled },
    shortcuts: {
      bindings: shortcuts.bindings.map((binding) => ({ ...binding })),
    },
  };
  return {
    ...momentsPreviewApi(),
    ...transcriptionPreviewApi(modelSeed),
    beginShortcutCapture: () => Promise.resolve(null),
    browseDefaultLocation: (kind) =>
      Promise.resolve(
        {
          project: "/Users/demo/Movies/Screenwide",
          recording: "/Users/demo/Recordings",
          screenshot: "/Users/demo/Screenshots",
        }[kind],
      ),
    endShortcutCapture: () => Promise.resolve(null),
    getAnnotateSettings: () => Promise.resolve(annotate),
    getGeneralSettings: () => Promise.resolve(general),
    getGlideSettings: () => Promise.resolve(glide),
    getOcrSettings: () => Promise.resolve(ocr),
    getRulerSettings: () => Promise.resolve(ruler),
    getShortcutDefaults: () => Promise.resolve(defaults),
    getShortcutSettings: () => Promise.resolve(shortcuts),
    hideSettings: () => Promise.resolve(null),
    listenToAnnotateSettings: () => Promise.resolve(() => undefined),
    minimize: () => Promise.resolve(),
    setAnnotateSettings: (next) => {
      annotate = next;
      return Promise.resolve(next);
    },
    setGeneralSettings: (next) => {
      general = next;
      return Promise.resolve(next);
    },
    setGlideSettings: (next) => {
      glide = next;
      return Promise.resolve(next);
    },
    setOcrSettings: (next) => {
      ocr = next;
      return Promise.resolve(next);
    },
    setRulerSettings: (next) => {
      ruler = next;
      return Promise.resolve(next);
    },
    setShortcutBinding: (action, shortcut) => {
      shortcuts = {
        bindings: [
          ...shortcuts.bindings.filter((binding) => binding.action !== action),
          { action, shortcut },
        ],
      };
      return Promise.resolve(shortcuts);
    },
  };
}

// What every Settings story shares: the window at its configured size, a
// settings API that keeps changes in memory, and a pane picked by the story.
export const settingsWindowMeta = {
  component: SettingsWindow,
  decorators: [featureStage({ window: "settings" })],
  parameters: {
    // Each pane is its own story, so the pane is not a control.
    controls: { exclude: ["initialSection", "updateSetting"] },
    layout: "fullscreen",
  },
  // A story picks where the models start through `parameters.modelSeed`,
  // since it is the preview's state rather than the window's props.
  render: function Render(args, { parameters }) {
    const [api] = useState(() =>
      createPreviewApi(
        isModelSeed(parameters.modelSeed) ? parameters.modelSeed : undefined,
      ),
    );
    const [checked, setChecked] = useState(false);
    return (
      <SettingsApiContext value={api}>
        <SettingsWindow
          initialQuery={args.initialQuery}
          initialSection={args.initialSection}
          updateSetting={
            args.updateSetting ?? (
              <SoftwareUpdateSetting
                currentVersion="0.1.0"
                error={null}
                onPress={() => {
                  setChecked(true);
                }}
                status={checked ? "up-to-date" : "idle"}
                updateVersion={null}
              />
            )
          }
        />
      </SettingsApiContext>
    );
  },
} satisfies Meta<typeof SettingsWindow>;
