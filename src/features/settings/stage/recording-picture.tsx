// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SettingsPreview } from "../preview/settings-preview";

import { useWallpaperSource } from "./use-wallpaper-source";

import type { SettingsPreviewScene } from "../preview/scenes/preview-scenes";

// A recording the compositor makes of the sample desktop, shown whole at
// the display's shape in the middle of the picture, as a recording is
// played back, rather than cut down to the picture's width. The room either
// side takes the desktop's colours, blurred and dimmed, so the recording
// sits in its own light rather than in an empty box.

export function RecordingPicture({ scene }: { scene: SettingsPreviewScene }) {
  const wallpaper = useWallpaperSource();
  return (
    <div className="absolute inset-0 flex justify-center p-inset-sm">
      <img
        alt=""
        className="absolute inset-0 size-full scale-110 object-cover opacity-40 blur-2xl"
        src={wallpaper}
      />
      <SettingsPreview className="h-full" scene={scene} />
    </div>
  );
}
