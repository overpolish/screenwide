// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type CSSProperties, type ReactNode, useRef } from "react";

import { ControlRow } from "../../../../composites/control-row/control-row";
import { platformArg, t } from "../../../../i18n/i18n";
import { Alert } from "../../../../primitives/alert/alert";
import { Button } from "../../../../primitives/button/button";
import { IslandParts } from "../../../../primitives/island/island-parts";
import { PillGroup } from "../../../../primitives/pill-group/pill-group";
import { ScrollArea } from "../../../../primitives/scroll-area/scroll-area";
import { Switch } from "../../../../primitives/switch/switch";

import { ExportCompare } from "./export-compare";
import { ExportLookChoice } from "./export-look-choice";
import { useExportBodyRoom } from "./use-export-body-room";

import type { CameraKey } from "../../../../bindings/CameraKey";
import type { ExportSubtitles } from "../../../../bindings/ExportSubtitles";
import type {
  ToolPanelExport,
  ToolPanelExportPatch,
} from "../../tools/tool-panel-export";

/** Under the file line, before the export: the picture as it comes out
 * beside it as drawn, where there is one to compare; how good the video
 * comes out, as one of four looks; then the options that apply to what is
 * exported. They are the Export island's exception to the islands' glyph
 * toggles: an export cannot be undone or previewed, so each option says in
 * words what it does, a `ControlRow` with a `Switch`, and a choice it
 * brings is a row of pills on the same line, before the switch, while the
 * option is on. Last is Export, or the last export's failure with Try again
 * in its place. Parts and rows come and go as options change, each its own
 * part (`IslandParts`). In a window too short for it all, the parts above
 * Export scroll, so Export stays in view (`useExportBodyRoom`). */
export function ExportOptions({
  onPatch,
  settings: shown,
}: {
  onPatch: (patch: ToolPanelExportPatch) => void;
  settings: ToolPanelExport;
}) {
  const bodyRef = useRef<HTMLDivElement>(null);
  const columnRef = useRef<HTMLDivElement>(null);
  const bodyRoom = useExportBodyRoom(bodyRef, columnRef);
  const hasSubtitles = shown.subtitles !== null && shown.subtitles !== "none";
  const keysCamera =
    shown.cameraOutput === "separate" && shown.cameraKey !== null;
  const switchRow = ({
    choice,
    isSelected,
    onChange,
    title,
  }: {
    isSelected: boolean;
    onChange: (isSelected: boolean) => void;
    title: string;
    /** Pills on the row, before its switch, where the option brings one. */
    choice?: ReactNode;
  }) => (
    <ControlRow key={title} title={title}>
      {(controlProps) => (
        <div className="flex items-center">
          <IslandParts className="flex items-center" direction="row">
            {choice ? (
              <div className="flex pe-separate" key="choice">
                {choice}
              </div>
            ) : null}
          </IslandParts>
          <Switch
            {...controlProps}
            isSelected={isSelected}
            onChange={onChange}
          />
        </div>
      )}
    </ControlRow>
  );
  return (
    <div className="flex flex-col gap-separate" ref={columnRef}>
      {/* Reaching a little past the section's inset on every side, its
          content held in by as much, so a control's focus ring at the edge
          is not cut off by the scrolled box. */}
      <div
        className="-m-1"
        ref={bodyRef}
        style={
          bodyRoom === undefined
            ? undefined
            : ({ "--export-body-room": bodyRoom } as CSSProperties)
        }
      >
        <ScrollArea
          className="p-1"
          constrainHeight
          overscroll="none"
          rootClassName="max-h-(--export-body-room)"
        >
          <IslandParts className="flex flex-col gap-separate">
            {shown.compare ? (
              <ExportCompare compare={shown.compare} key="compare" />
            ) : null}
            {shown.compare?.seeThrough ? (
              <Alert
                className="w-0 min-w-full px-inset-sm"
                color="warning"
                key="see-through"
                role="status"
              >
                {t("editor-export-jpeg-see-through")}
              </Alert>
            ) : null}
            {shown.quality ? (
              <ExportLookChoice
                key="looks"
                looks={shown.quality.looks}
                onChange={(look) => {
                  onPatch({ look });
                }}
                value={shown.quality.look}
              />
            ) : null}
            <IslandParts className="flex flex-col gap-inline" key="options">
              {shown.subtitles === null
                ? null
                : switchRow({
                    choice: hasSubtitles ? (
                      <PillGroup
                        aria-label={t("editor-export-subtitles-format")}
                        items={[
                          {
                            id: "srt",
                            label: t("editor-export-subtitles-srt"),
                          },
                          {
                            id: "vtt",
                            label: t("editor-export-subtitles-vtt"),
                          },
                        ]}
                        key="format"
                        onSelectionChange={(id) => {
                          onPatch({ subtitles: id as ExportSubtitles });
                        }}
                        selected={shown.subtitles}
                      />
                    ) : null,
                    isSelected: hasSubtitles,
                    onChange: (isOn) => {
                      onPatch({ subtitles: isOn ? "srt" : "none" });
                    },
                    title: t("editor-export-subtitles"),
                  })}
              {shown.cameraOutput === null
                ? null
                : switchRow({
                    choice: keysCamera ? (
                      <PillGroup
                        aria-label={t("editor-export-camera-key")}
                        items={[
                          {
                            id: "green",
                            label: t("editor-export-camera-key-green"),
                          },
                          {
                            id: "blue",
                            label: t("editor-export-camera-key-blue"),
                          },
                        ]}
                        key="key"
                        onSelectionChange={(id) => {
                          onPatch({ cameraKey: id as CameraKey });
                        }}
                        selected={shown.cameraKey ?? "green"}
                      />
                    ) : null,
                    isSelected: shown.cameraOutput === "separate",
                    onChange: (isOn) => {
                      onPatch({ cameraOutput: isOn ? "separate" : "combined" });
                    },
                    title: t("editor-export-camera-separate"),
                  })}
              {shown.collapseAudio === null
                ? null
                : switchRow({
                    isSelected: shown.collapseAudio,
                    onChange: (collapseAudio) => {
                      onPatch({ collapseAudio });
                    },
                    title: t("editor-export-mix-audio"),
                  })}
              {switchRow({
                isSelected: shown.revealWhenDone,
                onChange: (revealWhenDone) => {
                  onPatch({ revealWhenDone });
                },
                title: t("editor-export-reveal-when-done", {
                  platform: platformArg(),
                }),
              })}
              {switchRow({
                isSelected: shown.deleteProjectAfterExport,
                onChange: (deleteProjectAfterExport) => {
                  onPatch({ deleteProjectAfterExport });
                },
                title: t("editor-export-delete-after"),
              })}
            </IslandParts>
          </IslandParts>
        </ScrollArea>
      </div>
      <IslandParts>
        {shown.failure === null ? (
          <div className="flex justify-end" key="export">
            <Button
              color="primary"
              isDisabled={!shown.canExport}
              onPress={() => {
                onPatch({ export: true });
              }}
            >
              {t("editor-export-action")}
            </Button>
          </div>
        ) : (
          <Alert
            action={
              <Button
                isDisabled={!shown.canExport}
                onPress={() => {
                  onPatch({ export: true });
                }}
              >
                {t("editor-export-try-again")}
              </Button>
            }
            className="min-h-island-control ps-inset-sm"
            color="error"
            key="failure"
            role="alert"
          >
            {t("editor-export-failed", { reason: shown.failure })}
          </Alert>
        )}
      </IslandParts>
    </div>
  );
}
