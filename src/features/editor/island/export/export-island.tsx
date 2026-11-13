// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CircleCheck, Upload } from "lucide-react";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { appLocale, t } from "../../../../i18n/i18n";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";
import { Text } from "../../../../primitives/text/text";
import { formatEta } from "../../duration";
import { IslandWorkLine } from "../island-work-line";

import { ExportFileLine } from "./export-file-line";
import {
  ExportFileChip,
  ExportFinishActions,
  type ExportFinishHandlers,
} from "./export-finish";
import { ExportOptions } from "./export-options";

import type {
  ToolPanelExport,
  ToolPanelExportPatch,
  ToolPanelExportProgress,
} from "../../tools/tool-panel-export";
import type { EditorIslandContent } from "../editor-island";

// Export, brought out by the title bar's button over the picture, whose
// editor stands still under it. Three things in turn, each a row of its
// own that clears into the next: the export to set up, the file first in
// the section and its quality and options beneath; the export at work, with
// Cancel; and the file written, with what can be done with it. A failure
// comes back to the first, with Try again.

export type ExportIslandProps = {
  export: ToolPanelExport;
  finish: ExportFinishHandlers;
  onPatch: (patch: ToolPanelExportPatch) => void;
};

/** What the save is writing now, in the user's terms. */
const workTitle = (
  kind: ToolPanelExport["kind"],
  phase: ToolPanelExportProgress["phase"],
) => {
  if (kind === "audio") return t("editor-export-saving-audio");
  if (kind === "screenshot") return t("editor-export-saving-screenshot");
  if (phase === "camera") return t("editor-export-saving-camera");
  if (phase === "finalizing") return t("editor-export-finalizing");
  return t("editor-export-saving-recording");
};

/** How long is left once it can be told, else how far it has got. */
const workNote = ({ etaSeconds, percent }: ToolPanelExportProgress) => {
  if (etaSeconds !== null) return formatEta(etaSeconds);
  if (percent === null) return t("editor-export-starting");
  return new Intl.NumberFormat(appLocale().formatLocale, {
    style: "percent",
  }).format(percent / 100);
};

export function exportIsland({
  export: settings,
  finish,
  onPatch,
}: ExportIslandProps): EditorIslandContent {
  const { exported, progress } = settings;
  if (exported) {
    return {
      actions: <ExportFinishActions {...finish} />,
      key: "export-done",
      section: (
        <div className="flex flex-col gap-inline">
          <div className="flex">
            <ExportFileChip
              file={exported}
              kind={settings.kind}
              onDragOut={finish.onDragOut}
            />
          </div>
          {exported.credits ? (
            // It wraps to the width the file sets rather than widening the
            // island with the credits file's name.
            <Text
              className="w-0 min-w-full px-inset-sm text-fg-secondary"
              variant="caption"
            >
              {t("editor-export-credits", { name: exported.credits })}
            </Text>
          ) : null}
        </div>
      ),
      title: (
        <IslandRowTitle
          glyph={<CircleCheck aria-hidden className="size-icon" />}
          title={t("editor-export-done")}
        />
      ),
    };
  }
  if (progress) {
    return {
      actions: progress.canCancel ? (
        <ConfirmActionButton
          confirmLabel={t("editor-export-confirm-cancel")}
          isDisabled={progress.isCanceling}
          label={
            progress.isCanceling
              ? t("editor-export-canceling")
              : t("editor-export-cancel")
          }
          onConfirm={() => {
            onPatch({ cancel: true });
          }}
        />
      ) : undefined,
      key: "export-work",
      section: (
        <IslandWorkLine
          label={workNote(progress)}
          value={progress.percent === null ? null : progress.percent / 100}
        />
      ),
      title: (
        <IslandRowTitle
          glyph={<Upload aria-hidden className="size-icon" />}
          title={workTitle(settings.kind, progress.phase)}
        />
      ),
    };
  }
  return {
    key: "export",
    section: (
      <div className="flex flex-col gap-separate">
        <ExportFileLine
          directory={settings.directory}
          extension={settings.extension}
          fileStem={settings.fileStem}
          onBrowse={() => {
            onPatch({ browse: true });
          }}
          onFileStemChange={(fileStem) => {
            onPatch({ fileStem });
          }}
          onRecordingFormatChange={(recordingFormat) => {
            onPatch({ recordingFormat });
          }}
          onScreenshotFormatChange={(screenshotFormat) => {
            onPatch({ screenshotFormat });
          }}
          recordingFormat={settings.recordingFormat}
          screenshotFormat={settings.screenshotFormat}
        />
        <ExportOptions onPatch={onPatch} settings={settings} />
      </div>
    ),
    title: (
      <IslandRowTitle
        glyph={<Upload aria-hidden className="size-icon" />}
        title={
          settings.kind === "audio"
            ? t("editor-export-title-audio")
            : settings.kind === "screenshot"
              ? t("editor-export-title-screenshot")
              : t("editor-export-title-recording")
        }
      />
    ),
  };
}
