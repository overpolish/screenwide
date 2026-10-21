// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PathField } from "../../../composites/path-field/path-field";
import { t } from "../../../i18n/i18n";
import { Button } from "../../../primitives/button/button";
import { Checkbox } from "../../../primitives/checkbox/checkbox";
import { CircularProgress } from "../../../primitives/circular-progress/circular-progress";
import { TextField } from "../../../primitives/input-fields/text-field";
import { PillGroup } from "../../../primitives/pill-group/pill-group";
import { formatBytes } from "../duration";

import {
  CompressionSelect,
  ExportRow,
  SubtitlesRow,
} from "./editor-export-dialog-rows";
import { EditorExportFormProps } from "./editor-export-form-props";
import {
  cameraResolutionItems,
  outputResolutionItems,
} from "./export-resolution-items";
import { ExportResolutionSelect } from "./export-resolution-select";

/** The host owns the settings; this only presents and reports them. */
export function EditorExportForm({
  artifact,
  cameraCompression = 0,
  cameraOutput = null,
  cameraResolutionScalePercent = 100,
  canExport = true,
  collapseAudio = false,
  compression = 0,
  deleteProjectAfterExport = false,
  directory,
  enabledAudioTrackCount = 0,
  estimatedSizeBytes,
  extension,
  fileStem,
  hasTranscript = false,
  includeCamera = false,
  isEstimatingSize,
  isSaving,
  onBrowse,
  onCameraCompressionChange,
  onCameraResolutionScaleChange,
  onCancel,
  onCollapseAudioChange,
  onCompressionChange,
  onDeleteProjectAfterExportChange,
  onExport,
  onFileStemChange,
  onOpenLocationAfterExportChange,
  onResolutionScaleChange,
  onSubtitlesChange,
  openLocationAfterExport = false,
  recordingOutput,
  resolutionScalePercent,
  subtitles = "none",
}: EditorExportFormProps) {
  if (!artifact) return null;
  const recording = artifact.kind === "recording" ? artifact : null;
  const camera =
    recording && includeCamera && cameraOutput !== "combined"
      ? recording.camera
      : null;
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
  const estimate =
    typeof estimatedSizeBytes === "number"
      ? formatBytes(estimatedSizeBytes)
      : t("editor-export-unavailable");

  return (
    <div className="flex flex-col gap-layout outline-none">
      <div className="grid grid-cols-[max-content_minmax(0,1fr)] gap-x-section gap-y-section">
        <ExportRow title={t("editor-export-file-name")}>
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
        <ExportRow title={t("editor-export-save-to")}>
          {() => (
            <PathField
              aria-label={t("editor-export-save-to")}
              className="w-full"
              fullWidth
              isDisabled={isSaving}
              onBrowse={onBrowse}
              value={directory}
            />
          )}
        </ExportRow>
        {/* One format for now: choosing another is a backend follow-up. */}
        <ExportRow layout="end" title={t("editor-export-format")}>
          {() => (
            <PillGroup
              aria-label={t("editor-export-format")}
              isDisabled
              items={[{ id: extension, label: extension.toUpperCase() }]}
              onSelectionChange={() => undefined}
              selected={extension}
            />
          )}
        </ExportRow>
        {showResolution ? (
          <ExportRow title={t("editor-export-output-size")}>
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
          <ExportRow title={t("editor-export-camera-output-size")}>
            {(props) => (
              <ExportResolutionSelect
                {...props}
                id="export-camera-size"
                isDisabled={isSaving}
                // A separate camera file is saved at the camera's own
                // framing, whatever its canvas was given while it was a pane.
                items={cameraResolutionItems(
                  camera,
                  cameraOutput === "separate"
                    ? undefined
                    : recordingOutput?.camera,
                )}
                onChange={(value) => {
                  onCameraResolutionScaleChange?.(Number(value));
                }}
                value={cameraResolutionScalePercent.toString()}
              />
            )}
          </ExportRow>
        ) : null}
        {showCompression ? (
          <ExportRow title={t("editor-export-compression")}>
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
          <ExportRow title={t("editor-export-camera-compression")}>
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
            description={t("editor-export-collapse-audio-description")}
            layout="trailing"
            title={t("editor-export-collapse-audio")}
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
        {recording ? (
          <SubtitlesRow
            hasTranscript={hasTranscript}
            isDisabled={isSaving}
            onChange={onSubtitlesChange}
            value={subtitles}
          />
        ) : null}
        <ExportRow layout="trailing" title={t("editor-export-open-folder")}>
          {(props) => (
            <Checkbox
              {...props}
              isDisabled={isSaving}
              isSelected={openLocationAfterExport}
              onChange={onOpenLocationAfterExportChange}
            />
          )}
        </ExportRow>
        <ExportRow
          // A screenshot is also copied from the editor, which the choice
          // covers as well.
          description={
            recording
              ? t("editor-export-delete-project-description")
              : t("editor-export-delete-project-copy-description")
          }
          layout="trailing"
          title={t("editor-export-delete-project")}
        >
          {(props) => (
            <Checkbox
              {...props}
              isDisabled={isSaving}
              isSelected={deleteProjectAfterExport}
              onChange={onDeleteProjectAfterExportChange}
            />
          )}
        </ExportRow>
      </div>
      <div className="flex items-center justify-end gap-control-inset">
        {recording ? (
          <p className="text-body text-fg-secondary me-auto flex items-center gap-control-inset tabular-nums text-content-fg-secondary">
            {isEstimatingSize ? (
              <>
                {t("editor-export-estimated-size-label")}
                <CircularProgress
                  aria-label={t("editor-export-estimating")}
                  isIndeterminate
                  size="small"
                />
              </>
            ) : (
              t("editor-export-estimated-size", { size: estimate })
            )}
          </p>
        ) : null}
        <Button onPress={onCancel}>{t("editor-export-cancel")}</Button>
        <Button color="primary" isDisabled={!canExport} onPress={onExport}>
          {t("editor-export")}
        </Button>
      </div>
    </div>
  );
}
