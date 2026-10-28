// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion } from "motion/react";

import { ControlRow } from "../../../../composites/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { CircularProgress } from "../../../../primitives/circular-progress/circular-progress";
import { NumberField } from "../../../../primitives/input-fields/number-field";
import { focusIn } from "../../../../primitives/island/island-motion";
import { Switch } from "../../../../primitives/switch/switch";
import { IslandFigure } from "../island-figure";
import { IslandRotation } from "../island-rotation";

import type { SceneFramingProps, ScenePane } from "./scene-framing";

// A pane's group: exact figures for what is mostly done by hand on the
// picture (how far it zooms in, how round its corners are, which part of it
// sits in the middle of its box), with the camera's face following and the
// screen's turn in depth. A figure off its own default shows a reset at its
// start, which goes again once it is back.

const MAX_ZOOM_PERCENT = 800;

const percent = (share: number) => Math.round(share * 100);

export function ScenePaneGroup(
  props: SceneFramingProps & { pane: "camera" | "screen" },
) {
  const { camera, onCameraChange, onPaneReset, onScreenChange, pane, screen } =
    props;
  const framing: ScenePane | null = pane === "camera" ? camera : screen;
  if (!framing) return null;
  const change = pane === "camera" ? onCameraChange : onScreenChange;
  return (
    // As wide as its rows need. Rows sit as close as the island's own
    // parts do.
    <div className="flex flex-col gap-inline">
      <ControlRow title={t("editor-island-scene-zoom")}>
        {(controlProps) => (
          <IslandFigure
            controlProps={controlProps}
            isDefault={framing.zoom === 1}
            onReset={() => {
              onPaneReset(pane, "zoom");
            }}
            resetLabel={t("editor-island-scene-reset-zoom")}
          >
            <NumberField
              aria-label={t("editor-island-scene-zoom")}
              maxValue={MAX_ZOOM_PERCENT}
              minValue={100}
              onChange={(value) => {
                change({ zoom: value / 100 });
              }}
              rightSection="%"
              step={10}
              value={percent(framing.zoom)}
            />
          </IslandFigure>
        )}
      </ControlRow>
      <ControlRow title={t("editor-island-scene-radius")}>
        {(controlProps) => (
          <IslandFigure
            controlProps={controlProps}
            isDefault={!framing.hasOwnRadius}
            onReset={() => {
              onPaneReset(pane, "radius");
            }}
            resetLabel={t("editor-island-scene-reset-radius")}
          >
            <NumberField
              aria-label={t("editor-island-scene-radius")}
              maxValue={50}
              minValue={0}
              onChange={(radius) => {
                change({ radius });
              }}
              rightSection="%"
              step={1}
              value={framing.radius}
            />
          </IslandFigure>
        )}
      </ControlRow>
      <ControlRow title={t("editor-island-scene-position")}>
        {(controlProps) => (
          <IslandFigure
            controlProps={controlProps}
            isDefault={framing.focusX === 0.5 && framing.focusY === 0.5}
            onReset={() => {
              onPaneReset(pane, "position");
            }}
            resetLabel={t("editor-island-scene-reset-position")}
          >
            <NumberField
              aria-label={t("editor-island-scene-position-x")}
              leftSection={t("editor-island-scene-x")}
              maxValue={100}
              minValue={0}
              onChange={(value) => {
                change({ focusX: value / 100 });
              }}
              rightSection="%"
              step={1}
              value={percent(framing.focusX)}
            />
            <NumberField
              aria-label={t("editor-island-scene-position-y")}
              leftSection={t("editor-island-scene-y")}
              maxValue={100}
              minValue={0}
              onChange={(value) => {
                change({ focusY: value / 100 });
              }}
              rightSection="%"
              step={1}
              value={percent(framing.focusY)}
            />
          </IslandFigure>
        )}
      </ControlRow>
      {pane === "screen" && screen ? (
        <IslandRotation
          onChange={props.onRotationChange}
          rotation={screen.rotation}
        />
      ) : null}
      {pane === "camera" && camera ? (
        <ControlRow title={t("editor-island-scene-follow-face")}>
          {(controlProps) => (
            <>
              {/* While the camera looks for the face, a spinner beside the
                  switch that turned it on, coming into focus as the
                  island's parts do. */}
              <AnimatePresence initial={false}>
                {camera.followFace && camera.isFindingFace ? (
                  <motion.span
                    animate={focusIn.animate}
                    className="flex"
                    exit={focusIn.exit}
                    initial={focusIn.exit}
                    key="finding"
                    transition={focusIn.transition}
                  >
                    <CircularProgress
                      aria-label={t("editor-island-scene-finding-face")}
                      isIndeterminate
                      size="small"
                    />
                  </motion.span>
                ) : null}
              </AnimatePresence>
              <Switch
                {...controlProps}
                isSelected={camera.followFace}
                onChange={props.onFollowFaceChange}
              />
            </>
          )}
        </ControlRow>
      ) : null}
      {/* A one-shot action, at the end of its own row and as wide as its
          label, as the layout group's are. */}
      {pane === "camera" && camera?.followFace ? (
        <div className="flex justify-end">
          <Button onPress={props.onRecenter}>
            {t("editor-island-scene-recenter")}
          </Button>
        </div>
      ) : null}
    </div>
  );
}
