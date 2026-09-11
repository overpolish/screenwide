// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CircleDotDashed, Crop, MousePointer2, ScanSquare } from "lucide-react";
import { useState } from "react";

import { FeatureStoryStage } from "../../../storybook/feature-story-stage";
import { defaultRecordingOutput } from "../screenshot-output";

import { PreviewToolReset } from "./preview-tool-reset";
import { PreviewToolToggle } from "./preview-tool-toggle";
import { PreviewOutputSize, PreviewToolbar } from "./preview-toolbar";
import {
  RecordingCanvasTool,
  RecordingCanvasTools,
} from "./recording-crop-toggle";

import type { Meta, StoryObj } from "@storybook/react-vite";

const screenshotTools = [
  {
    icon: MousePointer2,
    label: "Select",
    name: "Select screenshot",
    shortcut: "V",
    tool: "select",
  },
  {
    icon: ScanSquare,
    label: "Resize canvas",
    name: "Resize canvas",
    shortcut: "F",
    tool: "canvas",
  },
  {
    icon: Crop,
    label: "Crop",
    name: "Crop screenshot",
    shortcut: "C",
    tool: "crop",
  },
  {
    icon: CircleDotDashed,
    label: "Recenter",
    name: "Recenter screenshot",
    shortcut: "R",
    tool: "recenter",
  },
] as const;

/** Local control state only: no native preview, output mutation or clipboard. */
function ToolbarPreview({
  initialTool = "select",
  isDisabled = false,
  kind = "screenshot",
  outputHeight = 1080,
  outputWidth = 1920,
  zoomPercent = 100,
}: {
  initialTool?: RecordingCanvasTool;
  isDisabled?: boolean;
  kind?: "screenshot" | "recording";
  outputHeight?: number;
  outputWidth?: number;
  zoomPercent?: number;
}) {
  const [tool, setTool] = useState<RecordingCanvasTool>(initialTool);

  return (
    <PreviewToolbar
      outputSize={
        <PreviewOutputSize height={outputHeight} width={outputWidth} />
      }
      tools={
        kind === "recording" ? (
          <RecordingCanvasTools
            activeTrack="primary"
            bakeCamera={false}
            isEnabled={!isDisabled}
            isRecenterEnabled={!isDisabled}
            onChange={() => undefined}
            onRecenterReset={() => undefined}
            onToolChange={setTool}
            outputs={defaultRecordingOutput({
              primary: { height: 1080, width: 1920 },
            })}
            screenPane={{
              height: 1080,
              kind: "screen",
              sourceHeight: 1080,
              sourceWidth: 1920,
              width: 1920,
              x: 0,
              y: 0,
            }}
            tool={tool}
          />
        ) : (
          <>
            {screenshotTools.map(
              ({ icon: Icon, label, name, shortcut, tool: value }) => (
                <PreviewToolToggle
                  isSelected={tool === value}
                  key={value}
                  label={label}
                  name={name}
                  onSelectedChange={(selected) => {
                    setTool(selected ? value : null);
                  }}
                  shortcut={shortcut}
                >
                  <Icon />
                </PreviewToolToggle>
              ),
            )}
            <PreviewToolReset onReset={() => undefined} tool={tool} />
          </>
        )
      }
      zoomPercent={zoomPercent}
    />
  );
}

const meta = {
  component: ToolbarPreview,
  decorators: [
    (Story, context) => (
      <FeatureStoryStage height={120} viewMode={context.viewMode} width={640}>
        <Story />
      </FeatureStoryStage>
    ),
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/Editor/Toolbar",
} satisfies Meta<typeof ToolbarPreview>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Screenshot: Story = {};
export const Recording: Story = { args: { kind: "recording" } };
export const RecordingToolsDisabled: Story = {
  args: { isDisabled: true, kind: "recording" },
};

export const NoToolSelected: Story = { args: { initialTool: null } };
export const CropSelected: Story = { args: { initialTool: "crop" } };
/** A workspace zoomed past its own pixels, with a portrait output size. */
export const ZoomedIn: Story = {
  args: { outputHeight: 2160, outputWidth: 1170, zoomPercent: 240 },
};
