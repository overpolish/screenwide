// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check, Paintbrush, Trash2 } from "lucide-react";
import { useEffect, useRef } from "react";
import { Button as AriaButton } from "react-aria-components";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { ControlRow } from "../../../../composites/control-row/control-row";
import { platformArg, t } from "../../../../i18n/i18n";
import { cn, elementFocusVisible, focusStyles } from "../../../../lib/styling";
import { Badge } from "../../../../primitives/badge/badge";
import { Button } from "../../../../primitives/button/button";
import { CircularProgress } from "../../../../primitives/circular-progress/circular-progress";
import { ScrollArea } from "../../../../primitives/scroll-area/scroll-area";
import { formatBytes } from "../../duration";
import { ToolPanelPatch } from "../tool-panel-patch";
import { ToolPanelCameraObjects } from "../tool-panel-store";

import { ToolProgress } from "./tool-progress";

type RowsProps = {
  change: (values: ToolPanelPatch) => void;
  isLocked: boolean;
  objects: ToolPanelCameraObjects;
};

/**
 * Objects on camera, picked on the preview: a click keeps one with the
 * person, an Option-click (Alt on Windows) removes one with the room. Each
 * is followed through the recording, its dot ringed with how far that has
 * got, and the dot goes to where it was picked.
 */
export function CameraObjectRows({ change, isLocked, objects }: RowsProps) {
  const { model } = objects;
  if (!model) return null;
  if (model.status === "downloaded") {
    return (
      <PickedObjects change={change} isLocked={isLocked} objects={objects} />
    );
  }
  const isDownloading = model.status === "downloading";
  return (
    <div>
      <ControlRow
        description={t("editor-panels-camera-objects-description")}
        title={t("editor-panels-camera-objects")}
        titleAccessory={<Badge>{formatBytes(model.sizeBytes)}</Badge>}
      >
        {(controlProps) => (
          <Button
            {...controlProps}
            isDisabled={isLocked || isDownloading}
            onPress={() => {
              change({ downloadCameraObjects: true });
            }}
          >
            {t("editor-panels-model-download")}
          </Button>
        )}
      </ControlRow>
      <ToolProgress
        isWorking={isDownloading}
        label={t("editor-panels-model-downloading")}
        value={model.progress}
      />
    </div>
  );
}

/** Not followed yet, nor being followed: waiting its turn. */
const isWaiting = (pick: ToolPanelCameraObjects["picks"][number]) =>
  !pick.followed && pick.following === null;

/** The objects picked, as dots to go to, the bin for the one chosen, and
 * picking on the preview turned on and off. Picking stops once the panel
 * goes, which would leave no Done to press. */
function PickedObjects({ change, isLocked, objects }: RowsProps) {
  const { chosen, isPicking, picks } = objects;
  const stopRef = useRef<() => void>(() => undefined);
  useEffect(() => {
    stopRef.current = () => {
      if (isPicking) change({ pickCameraObjects: false });
    };
  });
  useEffect(
    () => () => {
      stopRef.current();
    },
    [],
  );
  // The dot chosen, by a pick or on the preview, is scrolled into view.
  const stripRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (chosen === null) return;
    stripRef.current
      ?.querySelector(`[data-object="${String(chosen)}"]`)
      ?.scrollIntoView({ block: "nearest", inline: "nearest" });
  }, [chosen]);
  return (
    <div className="flex flex-col gap-control">
      <ControlRow
        controlClassName="min-w-0 flex-1 justify-end gap-control"
        title={t("editor-panels-camera-objects")}
      >
        {(controlProps) => (
          <div
            {...controlProps}
            className="flex min-w-0 flex-1 items-center justify-end gap-control"
            role="group"
          >
            {picks.length > 0 ? (
              <ScrollArea
                orientation="horizontal"
                rootClassName="min-w-0"
                scrollbarHidden
              >
                <div className="ms-auto flex w-max" ref={stripRef}>
                  {picks.map((pick) => (
                    <AriaButton
                      aria-label={
                        pick.keep
                          ? t("editor-panels-camera-object-kept")
                          : t("editor-panels-camera-object-removed")
                      }
                      className={cn(
                        "relative flex size-control-height cursor-default items-center justify-center rounded-control outline-none",
                        pick.id === chosen && "bg-fill",
                        focusStyles,
                        elementFocusVisible,
                      )}
                      data-object={pick.id}
                      isDisabled={isLocked}
                      key={pick.id}
                      onPress={() => {
                        change({ showCameraObject: pick.id });
                      }}
                    >
                      {pick.following !== null ? (
                        <span className="absolute inset-0 flex items-center justify-center">
                          <CircularProgress
                            aria-label={t(
                              "editor-panels-camera-object-following",
                            )}
                            size="small"
                            value={pick.following * 100}
                          />
                        </span>
                      ) : null}
                      {pick.painted ? (
                        <Paintbrush
                          aria-hidden
                          className={cn(
                            // Inside a progress ring's track while followed.
                            "size-control-compact",
                            pick.keep ? "text-success" : "text-error",
                            isWaiting(pick) && "opacity-50",
                          )}
                        />
                      ) : (
                        <span
                          className={cn(
                            "size-control-inset rounded-full",
                            pick.keep ? "bg-success" : "bg-error",
                            isWaiting(pick) && "opacity-50",
                          )}
                        />
                      )}
                    </AriaButton>
                  ))}
                </div>
              </ScrollArea>
            ) : null}
            {chosen !== null ? (
              <ConfirmActionButton
                armedIcon={<Check />}
                armedLabel={t("editor-panels-camera-object-confirm-remove")}
                idleIcon={<Trash2 />}
                idleLabel={t("editor-panels-camera-object-remove")}
                isDisabled={isLocked}
                onConfirm={() => {
                  change({ removeCameraObject: chosen });
                }}
              />
            ) : null}
            <Button
              isDisabled={isLocked}
              onPress={() => {
                change({ pickCameraObjects: !isPicking });
              }}
            >
              {isPicking
                ? t("editor-panels-camera-objects-done")
                : t("editor-panels-camera-objects-pick")}
            </Button>
          </div>
        )}
      </ControlRow>
      {isPicking ? (
        <p className="text-caption text-fg-secondary">
          {t("editor-panels-camera-objects-hint", { platform: platformArg() })}
        </p>
      ) : null}
      <ToolProgress
        isWorking={objects.working}
        label={t("editor-panels-camera-objects-working")}
        value={objects.progress}
      />
    </div>
  );
}
