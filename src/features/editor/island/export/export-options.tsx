// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { type CSSProperties, type ReactNode, useRef, useState } from "react";

import { ControlRow } from "../../../../composites/control-row/control-row";
import { platformArg, t } from "../../../../i18n/i18n";
import { Alert } from "../../../../primitives/alert/alert";
import { Button } from "../../../../primitives/button/button";
import { IslandFlow } from "../../../../primitives/island/island-flow";
import {
  arrive,
  focusIn,
  glide,
  reveal,
} from "../../../../primitives/island/island-motion";
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
 * in its place. Parts come and go as options change, the others gliding
 * (`IslandFlow`). In a window too short for it all, the parts above Export
 * scroll, so Export stays in view (`useExportBodyRoom`). */
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
          <RowChoice>{choice}</RowChoice>
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
          <IslandFlow className="flex flex-col gap-separate">
            {shown.compare ? (
              <ExportCompare compare={shown.compare} key="compare" />
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
            <div className="flex flex-col gap-inline" key="options">
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
            </div>
          </IslandFlow>
        </ScrollArea>
      </div>
      <IslandFlow>
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
      </IslandFlow>
    </div>
  );
}

/** A row's pills before its switch. Unlike `IslandFlow`, which lifts a part
 * leaving out of the flow, they stay in the row and fold their own width
 * away as they blur: the row is aligned to its end, so a lifted part was
 * pinned to a left edge sliding toward the switch and passed under it,
 * barely seen. The switch holds still either way. The pills are taller than
 * a switch's row, so their height folds with their width, but only between
 * the row's own height and theirs: folded from nothing, the row would sit at
 * its height while the first part of the fold passed and then change in a
 * blink at one end of it. */
function RowChoice({ children }: { children: ReactNode }) {
  const reduce = useReducedMotion();
  const still = { duration: 0 };
  const [rowHeight] = useState(islandRowHeight);
  return (
    <AnimatePresence initial={false}>
      {children ? (
        <motion.div
          animate={{ ...reveal.animate, height: "auto", width: "auto" }}
          className="flex items-center justify-end overflow-hidden"
          // The fade takes as long as the fold, so the pills are seen going
          // rather than gone before the row has closed up.
          exit={{
            ...focusIn.exit,
            height: rowHeight,
            transition: reduce ? still : glide,
            width: 0,
          }}
          initial={{ ...reveal.initial, height: rowHeight, width: 0 }}
          key="choice"
          transition={
            reduce ? still : { ...arrive, height: glide, width: glide }
          }
        >
          <div className="flex shrink-0 pe-separate">{children}</div>
        </motion.div>
      ) : null}
    </AnimatePresence>
  );
}

/** The island row's least height, in px, which the pills' height folds
 * down to: the `island-row` token as the page has it. */
function islandRowHeight() {
  const probe = document.createElement("div");
  probe.style.height = "var(--spacing-island-row)";
  document.body.append(probe);
  const height = probe.getBoundingClientRect().height;
  probe.remove();
  return height;
}
