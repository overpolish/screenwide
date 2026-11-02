// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { bestCameraMode } from "../../recording-inputs/camera-mode";
import { CameraThumbnail } from "../../recording-inputs/camera-thumbnail";
import {
  type CameraDevice,
  cameraRequestFps,
  type RecordingFps,
} from "../../recording-inputs/types";
import { useCameraPreview } from "../../recording-inputs/use-camera-preview";

// Anti-flicker's picture is the person's own camera (`usePreviewCamera`),
// asked for the cadences the setting picks, so turning it on steadies the
// picture they see. Without a camera the row has no picture.

export function CameraPicture({
  antiFlicker,
  camera,
  fps,
}: {
  antiFlicker: boolean;
  camera: CameraDevice;
  fps: RecordingFps;
}) {
  const mode = bestCameraMode(camera, cameraRequestFps(fps, antiFlicker));
  const preview = useCameraPreview({
    active: mode !== null,
    deviceId: camera.id,
    mode: mode ?? undefined,
    pal: antiFlicker,
  });
  return (
    <div className="absolute inset-0 flex justify-center p-inset-sm">
      <CameraThumbnail
        canvasRef={preview.canvasRef}
        className="h-full rounded-control"
        frameSize={preview.frameSize}
        hasFrame={preview.hasFrame}
        isStarting={preview.isStarting}
      />
    </div>
  );
}
