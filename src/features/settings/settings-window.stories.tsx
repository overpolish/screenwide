// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ContextType, useState } from "react";

import { FeatureStoryStage } from "../../storybook/feature-story-stage";

import { SettingsApiContext } from "./settings-api-context";
import { SettingsUpdateActions } from "./settings-update-actions";
import { SettingsWindow } from "./settings-window";

import type {
  GeneralSettings,
  GlideSettings,
  RulerSettings,
  OcrSettings,
  ShortcutDefaults,
  ShortcutSettings,
} from "./types";
import type { Meta, StoryObj } from "@storybook/react-vite";

function createPreviewApi(): ContextType<typeof SettingsApiContext> {
  let general: GeneralSettings = {
    launchAtLogin: false,
    openLocationAfterExport: true,
    recordScreenwideWindows: true,
    recordingCountdownSeconds: 3,
    recordingDirectory: null,
    screenshotDirectory: null,
    showRecordingBarOnLaunch: true,
    showRecordingConfidenceChecks: true,
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
      {
        action: "recognizeText",
        shortcut: "CommandOrControl+Shift+KeyT",
      },
      { action: "rulerOverlay", shortcut: "CommandOrControl+Shift+KeyR" },
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
  const defaults: ShortcutDefaults = {
    glide: { ...glide },
    ocr: { bindings: { ...ocr.bindings }, enabled: ocr.enabled },
    ruler: { bindings: { ...ruler.bindings }, enabled: ruler.enabled },
    shortcuts: {
      bindings: shortcuts.bindings.map((binding) => ({ ...binding })),
    },
  };
  return {
    beginShortcutCapture: () => Promise.resolve(null),
    browseDefaultLocation: (kind) =>
      Promise.resolve(
        `/Users/demo/${kind === "recording" ? "Recordings" : "Screenshots"}`,
      ),
    endShortcutCapture: () => Promise.resolve(null),
    getGeneralSettings: () => Promise.resolve(general),
    getGlideSettings: () => Promise.resolve(glide),
    getOcrSettings: () => Promise.resolve(ocr),
    getRulerSettings: () => Promise.resolve(ruler),
    getShortcutDefaults: () => Promise.resolve(defaults),
    getShortcutSettings: () => Promise.resolve(shortcuts),
    hideSettings: () => Promise.resolve(null),
    minimize: () => Promise.resolve(),
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

const meta = {
  component: SettingsWindow,
  decorators: [
    (Story, context) => (
      <FeatureStoryStage height={520} viewMode={context.viewMode} width={720}>
        <Story />
      </FeatureStoryStage>
    ),
  ],
  parameters: {
    controls: { exclude: ["updateActions"] },
    layout: "fullscreen",
  },
  render: function Render(args) {
    const [api] = useState(createPreviewApi);
    const [checked, setChecked] = useState(false);
    return (
      <SettingsApiContext value={api}>
        <SettingsWindow
          initialSection={args.initialSection}
          updateActions={
            args.updateActions ?? (
              <SettingsUpdateActions
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
  title: "Features/Settings",
} satisfies Meta<typeof SettingsWindow>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const UpdateAvailable: Story = {
  args: {
    updateActions: (
      <SettingsUpdateActions
        currentVersion="0.1.0"
        error={null}
        onPress={() => undefined}
        status="available"
        updateVersion="1.0.0"
      />
    ),
  },
};

export const Ruler: Story = { args: { initialSection: "ruler" } };
export const Ocr: Story = { args: { initialSection: "ocr" } };
