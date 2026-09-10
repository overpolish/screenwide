// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ArrowUpRight, X } from "lucide-react";

import { Button } from "../../../components/base/button/button";
import { Checkbox } from "../../../components/base/checkbox/checkbox";
import { TextField } from "../../../components/base/input-fields/text-field";
import { PillGroup } from "../../../components/base/pill-group/pill-group";
import { Text } from "../../../components/base/text/text";
import { PathField } from "../../../components/shared/path-field/path-field";
import { formatBytes } from "../duration";
import { RecordingOutputSettings } from "../screenshot-output";
import { EditorArtifact } from "../types";

import { CompressionSelect, ExportRow } from "./editor-export-dialog-rows";
import {
  cameraResolutionItems,
  outputResolutionItems,
} from "./export-resolution-items";
import { ExportResolutionSelect } from "./export-resolution-select";

export type EditorExportFormProps = {
  artifact: EditorArtifact | null;
  directory: string | null;
  /** What saving delivers, which is not always `artifact.extension`. */
  extension: string;
  fileStem: string;
  onBrowse: () => void;
  /** Dismisses without saving. The host owns what that means for its shell. */
  onCancel: () => void;
  onExport: () => void;
  onFileStemChange: (fileStem: string) => void;
  bakeCamera?: boolean;
  cameraCompression?: number;
  cameraResolutionScalePercent?: number;
  canExport?: boolean;
  collapseAudio?: boolean;
  compression?: number;
  enabledAudioTrackCount?: number;
  estimatedSizeBytes?: number | null;
  /** Whether the camera is one of the enabled video tracks. */
  includeCamera?: boolean;
  isEstimatingSize?: boolean;
  isSaving?: boolean;
  onCameraCompressionChange?: (compression: number) => void;
  onCameraResolutionScaleChange?: (scale: number) => void;
  onCollapseAudioChange?: (collapse: boolean) => void;
  onCompressionChange?: (compression: number) => void;
  onOpenLocationAfterExportChange?: (open: boolean) => void;
  onResolutionScaleChange?: (scale: number) => void;
  /** Reveals the saved file once it has been written. A stored preference. */
  openLocationAfterExport?: boolean;
  /** The canvases the editor's Dimensions setting gives the video and the
   * camera, which is what the size choices scale; absent, the source sizes
   * stand in. */
  recordingOutput?: RecordingOutputSettings | null;
  resolutionScalePercent?: number;
};

/** The host owns the settings; this only presents and reports them. */
export function EditorExportForm({
  artifact,
  bakeCamera = false,
  cameraCompression = 0,
  cameraResolutionScalePercent = 100,
  canExport = true,
  collapseAudio = false,
  compression = 0,
  directory,
  enabledAudioTrackCount = 0,
  estimatedSizeBytes,
  extension,
  fileStem,
  includeCamera = false,
  isEstimatingSize,
  isSaving,
  onBrowse,
  onCameraCompressionChange,
  onCameraResolutionScaleChange,
  onCancel,
  onCollapseAudioChange,
  onCompressionChange,
  onExport,
  onFileStemChange,
  onOpenLocationAfterExportChange,
  onResolutionScaleChange,
  openLocationAfterExport = false,
  recordingOutput,
  resolutionScalePercent,
}: EditorExportFormProps) {
  if (!artifact) return null;
  const recording = artifact.kind === "recording" ? artifact : null;
  const camera =
    recording && includeCamera && !bakeCamera ? recording.camera : null;
  const resolutionItems = recording
    ? outputResolutionItems(recording, recordingOutput?.primary)
    : [];
  // Shown whenever there is a picture, even with one choice: the row is
  // also where the output size is read.
  const showResolution =
    recording !== null &&
    recording.primaryKind !== "audio" &&
    resolutionItems.length > 0;
  const showCompression =
    recording !== null && recording.primaryKind !== "audio";
  const compressionDisabled = Boolean(isSaving) || !recording?.canCompress;
  const estimate = isEstimatingSize
    ? "Estimating"
    : typeof estimatedSizeBytes === "number"
      ? formatBytes(estimatedSizeBytes)
      : "Unavailable";

  return (
    <div className="flex flex-col gap-layout outline-none">
      <div className="grid grid-cols-[max-content_minmax(0,1fr)] gap-x-section gap-y-section">
        <ExportRow title="File name">
          {(props) => (
            <TextField
              {...props}
              autoFocus
              className="w-full"
              isDisabled={isSaving}
              onChange={onFileStemChange}
              value={fileStem}
            />
          )}
        </ExportRow>
        <ExportRow title="Save to">
          {() => (
            <PathField
              aria-label="Save to"
              className="w-full"
              fullWidth
              isDisabled={isSaving}
              onBrowse={onBrowse}
              value={directory}
            />
          )}
        </ExportRow>
        {/* One format for now: choosing another is a backend follow-up. */}
        <ExportRow title="Format">
          {() => (
            <PillGroup
              aria-label="Format"
              display="label"
              isDisabled
              items={[{ id: extension, label: extension.toUpperCase() }]}
              onSelectionChange={() => undefined}
              selected={extension}
            />
          )}
        </ExportRow>
        {showResolution ? (
          <ExportRow title="Output size">
            {(props) => (
              <ExportResolutionSelect
                {...props}
                id="export-size"
                isDisabled={isSaving}
                items={resolutionItems}
                onChange={(value) => {
                  onResolutionScaleChange?.(Number(value));
                }}
                value={(
                  resolutionScalePercent ?? Number(resolutionItems[0].id)
                ).toString()}
              />
            )}
          </ExportRow>
        ) : null}
        {camera ? (
          <ExportRow title="Camera output size">
            {(props) => (
              <ExportResolutionSelect
                {...props}
                id="export-camera-size"
                isDisabled={isSaving}
                items={cameraResolutionItems(camera, recordingOutput?.camera)}
                onChange={(value) => {
                  onCameraResolutionScaleChange?.(Number(value));
                }}
                value={cameraResolutionScalePercent.toString()}
              />
            )}
          </ExportRow>
        ) : null}
        {showCompression ? (
          <ExportRow title="Compression">
            {(props) => (
              <CompressionSelect
                {...props}
                id="export-quality"
                isDisabled={compressionDisabled}
                onChange={onCompressionChange}
                value={compression}
              />
            )}
          </ExportRow>
        ) : null}
        {camera ? (
          <ExportRow title="Camera compression">
            {(props) => (
              <CompressionSelect
                {...props}
                id="export-camera-quality"
                isDisabled={compressionDisabled}
                onChange={onCameraCompressionChange}
                value={cameraCompression}
              />
            )}
          </ExportRow>
        ) : null}
        {recording && recording.audioTracks.length > 1 ? (
          <ExportRow
            controlClassName="items-start py-1"
            description="Mix the selected tracks into one."
            title="Collapse audio tracks"
          >
            {(props) => (
              <Checkbox
                {...props}
                isDisabled={Boolean(isSaving) || enabledAudioTrackCount < 2}
                isSelected={collapseAudio}
                onChange={onCollapseAudioChange}
              />
            )}
          </ExportRow>
        ) : null}
        <ExportRow
          controlClassName="items-start py-1"
          title="Open folder after saving"
        >
          {(props) => (
            <Checkbox
              {...props}
              isDisabled={isSaving}
              isSelected={openLocationAfterExport}
              onChange={onOpenLocationAfterExportChange}
            />
          )}
        </ExportRow>
      </div>
      <div className="flex items-center justify-end gap-control-inset">
        {recording ? (
          <Text
            className="mr-auto tabular-nums text-content-fg-secondary"
            variant="subheadline"
          >
            Estimated size {estimate}
          </Text>
        ) : null}
        <Button onPress={onCancel}>
          <X />
          Cancel
        </Button>
        <Button color="primary" isDisabled={!canExport} onPress={onExport}>
          <ArrowUpRight />
          Export
        </Button>
      </div>
    </div>
  );
}
