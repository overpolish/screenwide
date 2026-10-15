// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { ToolPanelCameraEffects } from "../../tool-panels/tool-panel-store";
import { EditorArtifact } from "../../types";
import { useModel } from "../microphone/use-model";

import { usePublishedCameraBackground } from "./camera-background-channel";
import {
  CameraEffects,
  chooseRecordingCameraBackground,
  getRecordingCameraEffects,
  listenToCameraEffectsProgress,
  setRecordingCameraEffects,
} from "./camera-effects-api";
import { useCameraEffectsRevision } from "./use-camera-effects-revision";
import { CameraPicksTool, useCameraPicks } from "./use-camera-picks";

import type { CameraTool } from "../../../../bindings/CameraTool";

type Tool = keyof ToolPanelCameraEffects["progress"];
/** A change to some of the camera's effects, the rest kept as they are:
 * the background of the range the panel shows, or Enhance. */
export type CameraEffectsChange = Partial<
  Pick<ToolPanelCameraEffects, "background" | "enhance">
>;

export type CameraEffectsTool = {
  /** Give the camera these effects, the rest kept as they are. */
  change: (change: CameraEffectsChange) => void;
  /** Ask for a picture to put behind the person over the range the panel
   * shows, and show it there. */
  chooseBackground: () => void;
  /** Download the model background removal needs. */
  download: () => void;
  /** The objects picked to fix the matte, once it is made. */
  objects: CameraPicksTool | null;
  shown: ToolPanelCameraEffects;
};

type Held = {
  artifactId: number;
  effects: CameraEffects;
  /** Whether Enhance's last turn is still being carried out. */
  pending: boolean;
};

const TOOLS: Tool[] = ["background", "colour"];

/**
 * The camera's effects for the recording open, where it has a camera or is
 * of the camera alone. The background is the timeline's: the panel shows
 * the range chosen in the camera's lane or under the playhead, and changes
 * it there. Enhance is kept beside the edit. A change shows at once, and a
 * tool whose matte or measure is still to make shows its bar until the app
 * says it is.
 */
export function useCameraEffects(
  artifact: EditorArtifact | null,
): CameraEffectsTool | null {
  const artifactId =
    artifact?.kind === "recording" &&
    (artifact.camera || artifact.primaryKind === "camera")
      ? artifact.id
      : null;
  const model = useModel("cameraBackground");
  const isDownloaded = model.shown?.status === "downloaded";
  const background = usePublishedCameraBackground();
  const hasRanges = background?.shown.hasRanges ?? false;
  const range = background?.shown.range ?? null;
  const [held, setHeld] = useState<Held | null>(null);
  // Each change counts up, so an answer to one a later change has overtaken
  // is let go: a long one, such as measuring the colour, answers last.
  const latestRef = useRef(0);
  const [progress, setProgress] = useState<{
    artifactId: number;
    fractions: Partial<Record<CameraTool, number>>;
  } | null>(null);

  // Read again once the model arrives, as ranges come and go, and as the
  // camera is drawn differently: where the background is taken away, asking
  // starts the work it still needs.
  const revision = useCameraEffectsRevision(artifactId ?? -1);
  useEffect(() => {
    if (artifactId === null) return;
    let current = true;
    const ticket = latestRef.current;
    getRecordingCameraEffects(artifactId)
      .then((effects) => {
        if (current && ticket === latestRef.current)
          setHeld({ artifactId, effects, pending: false });
      })
      .catch((cause: unknown) => {
        console.error("Could not read the camera's effects", cause);
      });
    return () => {
      current = false;
    };
  }, [artifactId, isDownloaded, hasRanges, revision]);

  useEffect(() => {
    if (artifactId === null) return;
    let stop: (() => void) | undefined;
    let current = true;
    listenToCameraEffectsProgress(({ artifactId: id, fraction, tool }) => {
      if (id !== artifactId) return;
      setProgress((previous) => ({
        artifactId,
        fractions: {
          ...(previous?.artifactId === artifactId ? previous.fractions : {}),
          [tool]: fraction,
        },
      }));
    })
      .then((unlisten) => {
        if (current) stop = unlisten;
        else unlisten();
      })
      .catch((cause: unknown) => {
        console.error("Could not follow the camera's effects", cause);
      });
    return () => {
      current = false;
      stop?.();
    };
  }, [artifactId]);

  const fractions =
    progress?.artifactId === artifactId ? progress.fractions : {};
  const isHeld = artifactId !== null && held?.artifactId === artifactId;
  const backgroundFraction = fractions.background;
  // Picking objects reads the matte of the whole camera.
  const isMatted =
    isHeld &&
    isDownloaded &&
    hasRanges &&
    held.effects.canPickObjects &&
    (backgroundFraction === undefined || backgroundFraction >= 1);
  const objects = useCameraPicks(artifactId, isMatted, fractions.objects);
  if (artifactId === null || !isHeld) return null;
  const { effects } = held;
  const settle = (request: Promise<CameraEffects>, enhance: boolean) => {
    if (enhance) {
      const { colour: _colour, ...rest } = fractions;
      setProgress({ artifactId, fractions: rest });
    }
    setHeld({
      artifactId,
      effects: { ...effects, enhance },
      pending: enhance,
    });
    latestRef.current += 1;
    const ticket = latestRef.current;
    request
      .then((answered) => {
        if (ticket === latestRef.current)
          setHeld({ artifactId, effects: answered, pending: false });
      })
      .catch((cause: unknown) => {
        if (ticket !== latestRef.current) return;
        console.error("Could not change the camera's effects", cause);
        setHeld(held);
      });
  };
  const isOn = (tool: Tool) =>
    tool === "background" ? hasRanges : effects.enhance;
  const byTool = <Value>(value: (tool: Tool) => Value) =>
    Object.fromEntries(TOOLS.map((tool) => [tool, value(tool)])) as Record<
      Tool,
      Value
    >;
  return {
    change: (change) => {
      if (change.background !== undefined)
        background?.controls.setBackground(change.background);
      if (change.enhance !== undefined)
        settle(
          setRecordingCameraEffects(artifactId, change.enhance),
          change.enhance,
        );
    },
    chooseBackground: () => {
      if (!range) return;
      const id = range.id;
      chooseRecordingCameraBackground(artifactId, id)
        .then((answered) => {
          setHeld((current) =>
            current?.artifactId === artifactId
              ? { ...current, effects: answered }
              : current,
          );
          if (answered.pictures.includes(id))
            background?.controls.setBackground("image");
        })
        .catch((cause: unknown) => {
          console.error("Could not choose the camera's background", cause);
        });
    },
    download: model.download,
    objects,
    shown: {
      background: range?.background ?? "kept",
      enhance: effects.enhance,
      hasImage: range !== null && effects.pictures.includes(range.id),
      model: model.shown,
      objects: objects?.shown ?? null,
      progress: byTool((tool) => fractions[tool] ?? 0),
      working: byTool((tool) => {
        const fraction = fractions[tool];
        return (
          fraction !== undefined &&
          fraction < 1 &&
          ((tool === "colour" && held.pending) || isOn(tool))
        );
      }),
    },
  };
}
