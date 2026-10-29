// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Paintbrush, Trash2 } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useRef } from "react";

import { useTileMotion } from "../../../../composites/color-grid/tile-motion";
import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { ControlRow } from "../../../../composites/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { Button } from "../../../../primitives/button/button";
import { IconToggleButton } from "../../../../primitives/button/icon-button";
import { CircularProgress } from "../../../../primitives/circular-progress/circular-progress";
import { IslandFlow } from "../../../../primitives/island/island-flow";
import { ScrollArea } from "../../../../primitives/scroll-area/scroll-area";

import type { ToolPanelCameraObjects } from "../../tool-panels/tool-panel-selection";

type Pick = ToolPanelCameraObjects["picks"][number];

/** Not followed yet, nor being followed: waiting its turn. */
const isWaiting = (pick: Pick) => !pick.followed && pick.following === null;

/** One object picked: a dot, green kept with the person or red removed with
 * the room, or a brush where a line was painted over it; faint while it
 * waits to be followed, and ringed with how far following has got. */
function PickMark({ pick }: { pick: Pick }) {
  const color = pick.keep ? "text-success" : "text-error";
  return (
    <span className="relative flex size-full items-center justify-center">
      {pick.following === null ? null : (
        <span className="absolute inset-0 flex items-center justify-center">
          <CircularProgress
            aria-label={t("editor-panels-camera-object-following")}
            size="small"
            value={pick.following * 100}
          />
        </span>
      )}
      {pick.painted ? (
        <Paintbrush
          aria-hidden
          className={cn("size-icon-sm", color, isWaiting(pick) && "opacity-50")}
        />
      ) : (
        <span
          className={cn(
            "size-2.5 rounded-full",
            pick.keep ? "bg-success" : "bg-error",
            isWaiting(pick) && "opacity-50",
          )}
        />
      )}
    </span>
  );
}

/** The objects picked on the preview to put right where the cutout has them
 * wrong, as marks to go to, the bin for the one chosen, and picking turned
 * on and off. Picking stops once the island goes, which would leave no Done
 * to press. */
export function CameraObjectsLine({
  objects,
  onPick,
  onRemove,
  onShow,
}: {
  objects: ToolPanelCameraObjects;
  /** Turns picking on the preview on or off. */
  onPick: (isPicking: boolean) => void;
  onRemove: (id: number) => void;
  /** Goes to where an object was picked, choosing it. */
  onShow: (id: number) => void;
}) {
  const { chosen, isPicking, picks } = objects;
  const stopRef = useRef<() => void>(() => undefined);
  useEffect(() => {
    stopRef.current = () => {
      if (isPicking) onPick(false);
    };
  });
  useEffect(
    () => () => {
      stopRef.current();
    },
    [],
  );
  // The mark chosen, by a pick or on the preview, is scrolled into view.
  const stripRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (chosen === null) return;
    stripRef.current
      ?.querySelector(`[data-object="${String(chosen)}"]`)
      ?.scrollIntoView({ block: "nearest", inline: "nearest" });
  }, [chosen]);
  const { away, shown, slide } = useTileMotion();
  return (
    <ControlRow title={t("editor-panels-camera-objects")}>
      {(controlProps) => (
        <div {...controlProps} className="min-w-0" role="group">
          {/* The strip, the bin and Pick each come and go as objects are
              picked, chosen and removed, the others gliding to make room. */}
          <IslandFlow className="flex min-w-0 items-center gap-inline">
            {picks.length > 0 ? (
              // A few marks show at once; more scroll, so a long list of
              // picks never widens the island.
              <ScrollArea
                key="strip"
                orientation="horizontal"
                rootClassName="min-w-0 max-w-32"
                scrollbarHidden
              >
                {/* Positioned, so a mark on its way out is lifted out of the
                    strip here while the rest slide to their places, as kept
                    colours do in the colour grid. */}
                <div className="relative flex w-max" ref={stripRef}>
                  <AnimatePresence initial={false} mode="popLayout">
                    {picks.map((pick) => (
                      <motion.span
                        animate={shown}
                        className="flex"
                        exit={away}
                        initial={away}
                        key={pick.id}
                        {...slide}
                      >
                        <IconToggleButton
                          aria-label={
                            pick.keep
                              ? t("editor-panels-camera-object-kept")
                              : t("editor-panels-camera-object-removed")
                          }
                          data-object={pick.id}
                          isSelected={pick.id === chosen}
                          onChange={() => {
                            onShow(pick.id);
                          }}
                        >
                          <PickMark pick={pick} />
                        </IconToggleButton>
                      </motion.span>
                    ))}
                  </AnimatePresence>
                </div>
              </ScrollArea>
            ) : null}
            {chosen === null ? null : (
              <ConfirmActionButton
                confirmLabel={t("editor-panels-camera-object-confirm-remove")}
                icon={<Trash2 aria-hidden />}
                key="bin"
                label={t("editor-panels-camera-object-remove")}
                onConfirm={() => {
                  onRemove(chosen);
                }}
                variant="ghost"
              />
            )}
            <Button
              key="pick"
              onPress={() => {
                onPick(!isPicking);
              }}
            >
              {isPicking
                ? t("editor-panels-camera-objects-done")
                : t("editor-panels-camera-objects-pick")}
            </Button>
          </IslandFlow>
        </div>
      )}
    </ControlRow>
  );
}
