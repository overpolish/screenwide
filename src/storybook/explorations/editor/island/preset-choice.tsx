// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PillGroup } from "../../../../primitives/pill-group/pill-group";
import { type ScenePreset, SCENE_PRESETS } from "../editor-data";

// A scene's arrangement, as circles: each draws what it does to the frame,
// the screen as a grey pane and the camera as a round face.

const FRAME = { height: 40, opacity: 0.2, rx: 6, width: 64 };

export function PresetDrawing({ id }: { id: ScenePreset }) {
  return (
    <svg aria-hidden className="h-3.5 w-5.5" viewBox="0 0 64 40">
      <rect fill="currentColor" {...FRAME} />
      {id === "zoom" ? (
        <>
          <rect
            fill="currentColor"
            height={30}
            opacity={0.35}
            rx={3}
            width={54}
            x={5}
            y={5}
          />
          <rect
            fill="none"
            height={18}
            rx={3}
            stroke="currentColor"
            strokeWidth={4}
            width={28}
            x={26}
            y={14}
          />
        </>
      ) : null}
      {id === "side" ? (
        <>
          <rect
            fill="currentColor"
            height={28}
            opacity={0.6}
            rx={3}
            width={36}
            x={4}
            y={6}
          />
          <circle cx={52} cy={20} fill="currentColor" r={9} />
        </>
      ) : null}
      {id === "pip" || id === "screen" ? (
        <rect
          fill="currentColor"
          height={30}
          opacity={0.6}
          rx={3}
          width={54}
          x={5}
          y={5}
        />
      ) : null}
      {id === "pip" ? (
        <circle cx={50} cy={27} fill="currentColor" r={8} />
      ) : null}
      {id === "stacked" ? (
        <>
          <rect
            fill="currentColor"
            height={20}
            opacity={0.6}
            rx={3}
            width={40}
            x={12}
            y={3}
          />
          <circle cx={32} cy={31} fill="currentColor" r={7} />
        </>
      ) : null}
      {id === "camera" ? (
        <circle cx={32} cy={20} fill="currentColor" r={14} />
      ) : null}
    </svg>
  );
}

export function PresetChoice({
  onChange,
  value,
}: {
  onChange: (preset: ScenePreset) => void;
  value: ScenePreset;
}) {
  return (
    <PillGroup
      aria-label="Scene"
      items={SCENE_PRESETS.map(({ id, label }) => ({
        ariaLabel: label,
        icon: <PresetDrawing id={id} />,
        id,
      }))}
      onSelectionChange={(id) => {
        onChange(id as ScenePreset);
      }}
      selected={value}
    />
  );
}
