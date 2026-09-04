// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  AppWindowMac,
  AudioLines,
  Camera,
  Monitor,
  SquareDashed,
} from "lucide-react";

import {
  PillGroup,
  type PillGroupItem,
} from "../../../components/base/pill-group/pill-group";
import { RecordingMode } from "../../recording-sources/types";

const items: PillGroupItem[] = [
  { icon: <Monitor />, id: "screen", label: "Screen" },
  { icon: <SquareDashed />, id: "region", label: "Region" },
  { icon: <AppWindowMac />, id: "window", label: "Window" },
  { icon: <Camera />, id: "camera", label: "Camera only" },
  { icon: <AudioLines />, id: "audio", label: "Audio only" },
];

export function RecordingModePicker({
  isDisabled,
  mode,
  onChange,
}: {
  isDisabled: boolean;
  mode: RecordingMode;
  onChange: (mode: RecordingMode) => void;
}) {
  return (
    <PillGroup
      aria-label="Recording type"
      className="min-w-0 grow items-start justify-center self-stretch"
      display="icon"
      isDisabled={isDisabled}
      itemClassName="size-[70px] rounded-2xl [&_svg]:size-10!"
      items={items}
      onSelectionChange={(id) => {
        onChange(id as RecordingMode);
      }}
      selected={mode}
    />
  );
}
