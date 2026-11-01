// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useArgs } from "storybook/preview-api";
import { expect, fn, userEvent, waitFor, within } from "storybook/test";

import { t } from "../../../../i18n/i18n";
import { featureStage } from "../../../../storybook/feature-story-stage";
import { EditorIslandStage } from "../editor-island-stage";
import {
  editorIslandStoryMeta,
  type EditorIslandStoryProps,
} from "../editor-island-story-meta";

import { exportIsland } from "./export-island";
import {
  AUDIO,
  COMPARE_LOADING,
  MEASURING_QUALITY,
  PARTLY_MEASURED_QUALITY,
  SCREENSHOT,
  VIDEO,
} from "./export-story-fixtures";

import type {
  ToolPanelExport,
  ToolPanelExportPatch,
} from "../../tool-panels/tool-panel-export";
import type { Meta, StoryObj } from "@storybook/react-vite";

type ExportIslandStoryProps = EditorIslandStoryProps & {
  export: ToolPanelExport;
  onCopy: () => void;
  onDragOut: () => void;
  onPatch: (patch: ToolPanelExportPatch) => void;
  onReveal: () => void;
};

function ExportIsland({
  isShown,
  onCopy,
  onDragOut,
  onPatch,
  onReveal,
  ...props
}: ExportIslandStoryProps) {
  return (
    <EditorIslandStage
      content={exportIsland({
        export: props.export,
        finish: { onCopy, onDragOut, onReveal },
        onPatch,
      })}
      isShown={isShown}
    />
  );
}

/** What a setting asked for does to the story's export, as the editor
 * would do it. */
const patched = (
  shown: ToolPanelExport,
  patch: ToolPanelExportPatch,
): ToolPanelExport => {
  const quality = shown.quality && {
    ...shown.quality,
    look: patch.look ?? shown.quality.look,
  };
  return {
    ...shown,
    cameraKey: patch.cameraKey ?? shown.cameraKey,
    cameraOutput: patch.cameraOutput ?? shown.cameraOutput,
    collapseAudio: patch.collapseAudio ?? shown.collapseAudio,
    deleteProjectAfterExport:
      patch.deleteProjectAfterExport ?? shown.deleteProjectAfterExport,
    fileStem: patch.fileStem ?? shown.fileStem,
    quality,
    revealWhenDone: patch.revealWhenDone ?? shown.revealWhenDone,
    screenshotFormat: patch.screenshotFormat ?? shown.screenshotFormat,
    subtitles: patch.subtitles ?? shown.subtitles,
  };
};

const meta = {
  ...editorIslandStoryMeta,
  args: {
    export: VIDEO,
    isShown: true,
    onCopy: fn(),
    onDragOut: fn(),
    onPatch: fn(),
    onReveal: fn(),
  },
  component: ExportIsland,
  // A story may ask for a shorter canvas (`stageHeight`), as a short
  // editor window gives the island less room.
  decorators: [
    featureStage(({ parameters }) => ({
      height:
        typeof parameters.stageHeight === "number"
          ? parameters.stageHeight
          : 760,
      width: 720,
    })),
  ],
  // Every choice changes the story's export, as it would the editor's.
  render: function Render(args) {
    const [, updateArgs] = useArgs<ExportIslandStoryProps>();
    return (
      <ExportIsland
        {...args}
        onPatch={(patch) => {
          args.onPatch(patch);
          updateArgs({ export: patched(args.export, patch) });
        }}
      />
    );
  },
  title: "Features/Editor/Island/Export",
} satisfies Meta<typeof ExportIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

// The frame at the playhead compared across the divider, then the looks,
// four cards, each with its file's size.
export const Video: Story = {};

// The editor at its least height: the parts above Export scroll, fading at
// the edge with more, while the row and Export stay in view.
export const ShortWindow: Story = {
  parameters: { stageHeight: 480 },
};

// The sizes on their way: the cards stand at once, each with a quiet stub
// in place of its size until it lands.
export const Measuring: Story = {
  args: { export: { ...VIDEO, quality: MEASURING_QUALITY } },
};

// Some sizes landed and shown, the rest still stubs.
export const PartlyMeasured: Story = {
  args: { export: { ...VIDEO, quality: PARTLY_MEASURED_QUALITY } },
};

// The arrow keys move the choice from look to look.
export const ChooseWithArrows: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    const look = (name: string) =>
      canvas.getByRole("radio", { name: new RegExp(name) });
    look(t("editor-export-look-balanced")).focus();
    await userEvent.keyboard("{ArrowDown}");
    await waitFor(() =>
      expect(look(t("editor-export-look-original"))).toBeChecked(),
    );
    await userEvent.keyboard("{ArrowUp}{ArrowUp}");
    await waitFor(() =>
      expect(look(t("editor-export-look-smaller"))).toBeChecked(),
    );
  },
};

// The compared frame being made, softened under a spinner.
export const LoadingCompare: Story = {
  args: { export: { ...VIDEO, compare: COMPARE_LOADING } },
};

// The camera saved as a file of its own, its background taken away: the
// colour that fills it.
export const SeparateCameraGreen: Story = {
  args: {
    export: { ...VIDEO, cameraKey: "green", cameraOutput: "separate" },
  },
};

export const SeparateCameraBlue: Story = {
  args: {
    export: { ...VIDEO, cameraKey: "blue", cameraOutput: "separate" },
  },
};

// Without a transcript there is nothing to make subtitles of.
export const Untranscribed: Story = {
  args: { export: { ...VIDEO, subtitles: null } },
};

export const ManyAudioTracks: Story = {
  args: { export: { ...VIDEO, collapseAudio: true } },
};

export const AudioOnly: Story = { args: { export: AUDIO } };

export const ScreenshotPng: Story = { args: { export: SCREENSHOT } };

// A JPEG is compared with the screenshot as drawn; a PNG is written as
// drawn, so it has nothing to compare.
export const ScreenshotJpeg: Story = {
  args: {
    export: {
      ...SCREENSHOT,
      compare: VIDEO.compare,
      extension: "jpg",
      screenshotFormat: "jpeg",
    },
  },
};

export const Exporting: Story = {
  args: {
    export: {
      ...VIDEO,
      progress: {
        canCancel: true,
        etaSeconds: 95,
        isCanceling: false,
        percent: 42,
        phase: "recording",
      },
    },
  },
};

// Cancel asks twice: pressed once, it waits for the second press.
export const CancelConfirm: Story = {
  ...Exporting,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await userEvent.click(
      await canvas.findByRole("button", { name: t("editor-export-cancel") }),
    );
    await expect(
      canvas.getByRole("button", { name: t("editor-export-confirm-cancel") }),
    ).toBeVisible();
  },
};

export const Finished: Story = {
  args: {
    export: {
      ...VIDEO,
      exported: { bytes: 41_380_112, name: "Onboarding walkthrough.mp4" },
    },
  },
};

export const Failed: Story = {
  args: {
    export: {
      ...VIDEO,
      failure: "The folder cannot be written to.",
    },
  },
};
