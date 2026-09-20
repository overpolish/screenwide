// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Crop, MousePointer2, ScanSquare } from "lucide-react";

import { ButtonGroup } from "../../../components/base/button-group/button-group";
import { ToolToggle } from "../../../components/shared/tool-toggle/tool-toggle";
import { ANNOTATION_TOOLS } from "../tool-panels/tool-registry";

export type RecordingCanvasTool =
  "arrow" | "canvas" | "counter" | "crop" | "select" | null;

export function RecordingCanvasTools({
  isEnabled,
  isArrowEnabled = isEnabled,
  isFrameEnabled = isEnabled,
  isSelectEnabled = isEnabled,
  onToolChange,
  tool,
}: {
  isEnabled: boolean;
  onToolChange: (tool: RecordingCanvasTool) => void;
  tool: RecordingCanvasTool;
  isArrowEnabled?: boolean;
  isFrameEnabled?: boolean;
  isSelectEnabled?: boolean;
}) {
  return (
    <ButtonGroup aria-label="View tools" className="gap-control">
      {/* The marker is the anchor Select's panel hangs from: taking the tool
          up opens it, however the tool was taken up. */}
      <span className="inline-flex" data-editor-tool="select">
        <ToolToggle
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
        </ToolToggle>
      </span>
      <ToolToggle
        isDisabled={!isFrameEnabled}
        isSelected={tool === "canvas" && isFrameEnabled}
        label="Frame"
        name="Resize recording frame"
        onSelectedChange={(selected) => {
          onToolChange(selected ? "canvas" : null);
        }}
        shortcut="F"
      >
        <ScanSquare />
      </ToolToggle>
      <ToolToggle
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
      </ToolToggle>
      {ANNOTATION_TOOLS.map(({ icon: Icon, id, label, name, shortcut }) => (
        // The marker is the anchor this tool's panel hangs from.
        <span className="inline-flex" data-editor-tool={id} key={id}>
          <ToolToggle
            isDisabled={!isArrowEnabled}
            isSelected={tool === id && isArrowEnabled}
            label={label}
            name={name}
            onSelectedChange={(selected) => {
              onToolChange(selected ? id : null);
            }}
            shortcut={shortcut}
          >
            <Icon />
          </ToolToggle>
        </span>
      ))}
    </ButtonGroup>
  );
}
