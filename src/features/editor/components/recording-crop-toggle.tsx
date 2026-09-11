// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CircleDotDashed, Crop, MousePointer2, ScanSquare } from "lucide-react";

import { ButtonGroup } from "../../../components/base/button-group/button-group";

import { PreviewToolToggle } from "./preview-tool-toggle";

export type RecordingCanvasTool =
  "canvas" | "crop" | "recenter" | "select" | null;

export function RecordingCanvasTools({
  isEnabled,
  isFrameEnabled = isEnabled,
  isRecenterEnabled = false,
  isSelectEnabled = isEnabled,
  onToolChange,
  tool,
}: {
  isEnabled: boolean;
  onToolChange: (tool: RecordingCanvasTool) => void;
  tool: RecordingCanvasTool;
  isFrameEnabled?: boolean;
  isRecenterEnabled?: boolean;
  isSelectEnabled?: boolean;
}) {
  return (
    <ButtonGroup aria-label="View tools" className="gap-control">
      {/* The marker is the anchor Select's panel hangs from: taking the tool
          up opens it, however the tool was taken up. */}
      <span className="inline-flex" data-editor-tool="select">
        <PreviewToolToggle
          isDisabled={!isSelectEnabled}
          isSelected={tool === "select" && isSelectEnabled}
          label="Select"
          name="Select recording clip"
          onSelectedChange={(selected) => {
            onToolChange(selected ? "select" : null);
          }}
          shortcut="V"
        >
          <MousePointer2 />
        </PreviewToolToggle>
      </span>
      <PreviewToolToggle
        isDisabled={!isFrameEnabled}
        isSelected={tool === "canvas" && isFrameEnabled}
        label="Resize frame"
        name="Resize recording frame"
        onSelectedChange={(selected) => {
          onToolChange(selected ? "canvas" : null);
        }}
        shortcut="F"
      >
        <ScanSquare />
      </PreviewToolToggle>
      <PreviewToolToggle
        isDisabled={!isEnabled}
        isSelected={tool === "crop" && isEnabled}
        label="Crop"
        name="Crop recording clip"
        onSelectedChange={(selected) => {
          onToolChange(selected ? "crop" : null);
        }}
        shortcut="C"
      >
        <Crop />
      </PreviewToolToggle>
      <PreviewToolToggle
        isDisabled={!isRecenterEnabled}
        isSelected={tool === "recenter" && isRecenterEnabled}
        label="Recenter from current frame"
        name="Recenter recording from current frame"
        onSelectedChange={(selected) => {
          onToolChange(selected ? "recenter" : null);
        }}
        shortcut="R"
      >
        <CircleDotDashed />
      </PreviewToolToggle>
    </ButtonGroup>
  );
}
