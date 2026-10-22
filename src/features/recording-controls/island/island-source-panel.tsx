// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useNumberFormatter } from "react-aria";

import { t } from "../../../i18n/i18n";

import {
  ChosenRow,
  DisplayGrid,
  ModeCards,
  ModePicture,
  PromptRow,
} from "./island-source-parts";

import type {
  IslandDisplay,
  IslandInput,
  IslandMode,
  IslandRegion,
  IslandWindow,
} from "./island-types";

// What the island grows into to choose what is recorded: the modes along the
// top, and under them what that mode needs. A display is chosen from its
// picture. A window or region is pointed at on the screen itself; once it
// is, the window shows by its app and title, the region by its size and the
// displays it lies on. With no screen, it says which inputs are recorded.

/** Which inputs are on, as the No Screen sentence's variant. */
function inputsOn(camera: boolean, microphone: boolean, systemAudio: boolean) {
  if (camera && microphone && systemAudio) return "all";
  const on = [
    camera && "camera",
    microphone && "microphone",
    systemAudio && "system-audio",
  ].filter(Boolean);
  return on.length ? on.join("-") : "none";
}

export function SourcePanel({
  camera,
  chosenRegion,
  chosenWindow,
  displays,
  microphone,
  mode,
  onChooseRegion,
  onChooseWindow,
  onDisplayChange,
  onModeChange,
  selectedDisplayId,
  systemAudio,
}: {
  camera: IslandInput;
  displays: IslandDisplay[];
  microphone: IslandInput;
  mode: IslandMode;
  onModeChange: (mode: IslandMode) => void;
  selectedDisplayId: number;
  systemAudio: IslandInput;
  chosenRegion?: IslandRegion;
  chosenWindow?: IslandWindow;
  onChooseRegion?: () => void;
  onChooseWindow?: () => void;
  onDisplayChange?: (id: number) => void;
}) {
  // Pixels read as plain figures: no grouping, so 3200 is not 3,200.
  const pixels = useNumberFormatter({ useGrouping: false });
  return (
    <div className="flex w-[420px] flex-col gap-separate">
      <ModeCards mode={mode} onModeChange={onModeChange} />
      {mode === "screen" ? (
        <DisplayGrid
          displays={displays}
          onDisplayChange={onDisplayChange}
          selectedDisplayId={selectedDisplayId}
        />
      ) : mode === "window" ? (
        chosenWindow ? (
          <ChosenRow
            action={t("recording-controls-island-change-window")}
            detail={chosenWindow.appName}
            onAction={onChooseWindow}
            picture={
              chosenWindow.icon ? (
                <img
                  alt=""
                  className="size-8 shrink-0"
                  src={chosenWindow.icon}
                />
              ) : (
                <ModePicture mode="window" />
              )
            }
            title={chosenWindow.title}
          />
        ) : (
          <PromptRow
            action={t("recording-controls-island-choose-window")}
            onAction={onChooseWindow}
            prompt={t("recording-controls-island-pick-window")}
          />
        )
      ) : mode === "region" ? (
        chosenRegion ? (
          <ChosenRow
            action={t("recording-controls-island-change-region")}
            detail={
              chosenRegion.displays.length > 1
                ? t("recording-controls-island-region-displays", {
                    count: chosenRegion.displays.length,
                  })
                : (chosenRegion.displays[0] ?? "")
            }
            isTitleDirectional={false}
            onAction={onChooseRegion}
            picture={<ModePicture mode="region" />}
            title={t("recording-controls-island-region-size", {
              height: pixels.format(chosenRegion.height),
              width: pixels.format(chosenRegion.width),
            })}
          />
        ) : (
          <PromptRow
            action={t("recording-controls-island-choose-region")}
            onAction={onChooseRegion}
            prompt={t("recording-controls-island-pick-region")}
          />
        )
      ) : (
        <p className="rounded-control bg-fill px-inset py-separate text-body text-fg-secondary">
          {t("recording-controls-island-no-screen", {
            inputs: inputsOn(camera.isOn, microphone.isOn, systemAudio.isOn),
          })}
        </p>
      )}
    </div>
  );
}
