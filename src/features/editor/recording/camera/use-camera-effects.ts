// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { ToolPanelCameraEffects } from "../../tool-panels/tool-panel-store";
import { EditorArtifact } from "../../types";
import { useModel } from "../microphone/use-model";

import {
  CameraEffects,
  chooseRecordingCameraBackground,
  getRecordingCameraEffects,
  listenToCameraEffectsProgress,
  setRecordingCameraEffects,
} from "./camera-effects-api";
import { CameraPicksTool, useCameraPicks } from "./use-camera-picks";

import type { CameraTool } from "../../../../bindings/CameraTool";

type Tool = keyof ToolPanelCameraEffects["progress"];
/** A change to some of the camera's effects, the rest kept as they are. */
export type CameraEffectsChange = Partial<
  Pick<CameraEffects, "background" | "colour">
>;
type Change = CameraEffectsChange;

export type CameraEffectsTool = {
  /** Give the camera these effects, the rest kept as they are. */
  change: (change: Change) => void;
  /** Ask for a picture to put behind the person, and put it there. */
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
  /** The tools whose last turn the app is still carrying out. */
  pending: Tool[];
};

const TOOLS: Tool[] = ["background", "colour"];

const isOn = (effects: CameraEffects, tool: Tool) =>
  tool === "background" ? effects.background !== "kept" : effects[tool];

/** The tools `change` turns on, whose files may need making. */
const turnedOn = (change: Change): Tool[] =>
  TOOLS.filter((tool) =>
    tool === "background"
      ? change.background !== undefined && change.background !== "kept"
      : change[tool] === true,
  );

/**
 * The camera's effects for the recording open, where it has a camera or is
 * of the camera alone. A change shows at once, and a tool whose matte or
 * measure is still to make shows its bar until the app says it is.
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
  const [held, setHeld] = useState<Held | null>(null);
  // Each change counts up, so an answer to one a later change has overtaken
  // is let go: a long one, such as making the matte, answers last.
  const latestRef = useRef(0);
  const [progress, setProgress] = useState<{
    artifactId: number;
    fractions: Partial<Record<CameraTool, number>>;
  } | null>(null);

  // Read again once the model arrives: where background removal is on,
  // asking starts the work the recording still needs.
  useEffect(() => {
    if (artifactId === null) return;
    let current = true;
    const ticket = latestRef.current;
    getRecordingCameraEffects(artifactId)
      .then((effects) => {
        if (current && ticket === latestRef.current)
          setHeld({ artifactId, effects, pending: [] });
      })
      .catch((cause: unknown) => {
        console.error("Could not read the camera's effects", cause);
      });
    return () => {
      current = false;
    };
  }, [artifactId, isDownloaded]);

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
  const isMatted =
    isHeld &&
    isDownloaded &&
    held.effects.background !== "kept" &&
    !held.pending.includes("background") &&
    (backgroundFraction === undefined || backgroundFraction >= 1);
  const objects = useCameraPicks(artifactId, isMatted, fractions.objects);
  if (artifactId === null || !isHeld) return null;
  const { effects } = held;
  const settle = (
    request: Promise<CameraEffects>,
    pending: Tool[],
    optimistic: CameraEffects,
  ) => {
    setProgress({
      artifactId,
      fractions: Object.fromEntries(
        Object.entries(fractions).filter(
          ([tool]) => !pending.includes(tool as Tool),
        ),
      ),
    });
    setHeld({ artifactId, effects: optimistic, pending });
    latestRef.current += 1;
    const ticket = latestRef.current;
    request
      .then((answered) => {
        if (ticket === latestRef.current)
          setHeld({ artifactId, effects: answered, pending: [] });
      })
      .catch((cause: unknown) => {
        if (ticket !== latestRef.current) return;
        console.error("Could not change the camera's effects", cause);
        setHeld(held);
      });
  };
  const byTool = <Value>(value: (tool: Tool) => Value) =>
    Object.fromEntries(TOOLS.map((tool) => [tool, value(tool)])) as Record<
      Tool,
      Value
    >;
  return {
    change: (change) => {
      const next = { ...effects, ...change };
      settle(
        setRecordingCameraEffects(artifactId, next),
        turnedOn(change),
        next,
      );
    },
    chooseBackground: () => {
      settle(chooseRecordingCameraBackground(artifactId), ["background"], {
        ...effects,
      });
    },
    download: model.download,
    objects,
    shown: {
      background: effects.background,
      colour: effects.colour,
      hasImage: effects.hasImage,
      model: model.shown,
      objects: objects?.shown ?? null,
      progress: byTool((tool) => fractions[tool] ?? 0),
      working: byTool((tool) => {
        const fraction = fractions[tool];
        return (
          fraction !== undefined &&
          fraction < 1 &&
          (held.pending.includes(tool) || isOn(effects, tool))
        );
      }),
    },
  };
}
