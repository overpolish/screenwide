// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CameraOff, Mic, MicOff, Volume2, VolumeX } from "lucide-react";
import { useMemo } from "react";

import { ToggleMenuButton } from "../../../components/base/button/toggle-menu-button";
import { t } from "../../../i18n/i18n";
import { cn } from "../../../lib/styling";
import { CameraThumbnail } from "../../recording-inputs/camera-thumbnail";
import { useRecordingInputStore } from "../../recording-inputs/store";
import { RecordingInputs } from "../../recording-inputs/types";
import { useAudioPreview } from "../../recording-inputs/use-audio-preview";
import { useCameraPreview } from "../../recording-inputs/use-camera-preview";
import { RecordingMode } from "../../recording-sources/types";

import { useRecordingBarInputDevices } from "./recording-bar-input-devices";
import {
  inputBadge,
  inputMeter,
  warnedToggle,
} from "./recording-bar-input-parts";
import {
  closeIfOpen,
  showCameraPicker,
  showMicrophonePicker,
  showSystemAudioPicker,
} from "./recording-bar-input-pickers";
import {
  CAMERA_PANEL_ID,
  cameraName,
  cameraOptionOn,
  MICROPHONE_PANEL_ID,
  microphoneName,
  SYSTEM_AUDIO_PANEL_ID,
  systemAudioName,
} from "./recording-bar-input-state";

type RecordingBarInputsProps = {
  inputs: RecordingInputs;
  /** Whether the row is on screen and idle. The previews are native streams:
   * a dismissed bar stays mounted, so nothing but this stops them. */
  isPreviewActive: boolean;
  mode: RecordingMode;
  onInputChange: (input: keyof RecordingInputs, selected: boolean) => void;
  /** Why the selected camera cannot be recorded, shown as the camera's
   * warning badge and, on hover, its tooltip. */
  cameraWarning?: string;
  className?: string;
  isCameraLocked?: boolean;
  isDisabled?: boolean;
  isMicrophoneLocked?: boolean;
  microphoneWarning?: string;
  onCameraLockedPress?: () => void;
  onMicrophoneLockedPress?: () => void;
  systemAudioWarning?: string;
};

/** The bar's inputs group: what is being captured alongside the screen, each
 * input a toggle naming the device it is set to, which opens that input's
 * picker. */
export function RecordingBarInputs({
  cameraWarning,
  className,
  inputs,
  isCameraLocked = false,
  isDisabled = false,
  isMicrophoneLocked = false,
  isPreviewActive,
  microphoneWarning,
  mode,
  onCameraLockedPress,
  onInputChange,
  onMicrophoneLockedPress,
  systemAudioWarning,
}: RecordingBarInputsProps) {
  const {
    cameraFlippedById,
    cameraPalById,
    selectedCamera,
    selectedCameraMode,
    selectedMicrophone,
    selectedSystemAudio,
  } = useRecordingInputStore((state) => state);
  const { refreshAudioSources, refreshCameras, refreshMicrophones } =
    useRecordingBarInputDevices({
      isCameraLocked,
      isMicrophoneLocked,
      onInputChange,
    });

  const isAudioOnly = mode === "audio";
  const isCameraOnly = mode === "camera";
  const isCameraOn = isCameraOnly || (!isAudioOnly && inputs.camera);
  const cameraPal = cameraOptionOn(cameraPalById, selectedCamera);
  const cameraFlipped = cameraOptionOn(cameraFlippedById, selectedCamera);
  const selectedApplications = useMemo(
    () => selectedSystemAudio.filter((source) => source.kind === "application"),
    [selectedSystemAudio],
  );
  const selectedApplicationIds = useMemo(
    () => selectedApplications.map((source) => source.id),
    [selectedApplications],
  );
  const selectedProcessIds = useMemo(
    () => selectedApplications.flatMap((source) => source.processIds ?? []),
    [selectedApplications],
  );
  const capturesAllSystemAudio = selectedApplications.length === 0;

  // Nothing is streamed while an input is off: each preview is started only
  // for the toggle that is on, and stopped again the moment it goes off.
  const cameraPreview = useCameraPreview({
    active:
      isPreviewActive &&
      isCameraOn &&
      !isCameraLocked &&
      selectedCamera !== null &&
      selectedCameraMode !== null,
    deviceId: selectedCamera?.id,
    mode: selectedCameraMode ?? undefined,
    pal: cameraPal,
  });
  const microphonePreview = useAudioPreview({
    active: isPreviewActive && inputs.microphone && !isMicrophoneLocked,
    deviceId: selectedMicrophone?.id,
    kind: "microphone",
  });
  const systemAudioPreview = useAudioPreview({
    active: isPreviewActive && inputs.systemAudio,
    applicationIds: capturesAllSystemAudio ? undefined : selectedApplicationIds,
    kind: "system",
    processIds: capturesAllSystemAudio ? undefined : selectedProcessIds,
  });

  const cameraStageSize = cameraPreview.frameSize ?? selectedCameraMode;

  return (
    <div className={cn("flex items-center gap-control-inset", className)}>
      <div className="flex" data-popup-panel-trigger={CAMERA_PANEL_ID}>
        <ToggleMenuButton
          aria-label={t("recording-controls-camera")}
          badge={inputBadge(
            isCameraOn ? cameraWarning : undefined,
            isCameraLocked,
          )}
          isMenuDisabled={isDisabled || isAudioOnly}
          isSelected={isCameraOn}
          isToggleDisabled={isDisabled || isAudioOnly}
          label={cameraName(selectedCamera)}
          menuLabel={t("recording-controls-choose-camera")}
          off={<CameraOff />}
          onChange={(selected) => {
            void closeIfOpen(CAMERA_PANEL_ID);
            // Camera-only recording is the camera; the toggle shows that it
            // is on rather than offering to turn it off.
            if (isCameraOnly) return;
            if (isCameraLocked) onCameraLockedPress?.();
            else onInputChange("camera", selected);
          }}
          onMenuPress={(anchor, fromKeyboard) => {
            void showCameraPicker({
              anchor,
              focusContents: fromKeyboard,
              isCameraFlipped: cameraFlipped,
              isCameraOn,
              isCameraPal: cameraPal,
              refreshCameras,
              selectedCamera,
              selectedCameraMode,
            });
          }}
          renderToggle={warnedToggle(isCameraOn ? cameraWarning : undefined)}
          size="capture"
          variant="ghost"
        >
          {/* The thumbnail is the on glyph: the canvas stays mounted while the
              camera is on so the preview has somewhere to draw, and the glyph
              covers the seconds before the first frame arrives. It fills the
              glyph's box, which the part already insets from its edges. */}
          <span className="flex size-full items-center justify-center">
            <CameraThumbnail
              canvasRef={cameraPreview.canvasRef}
              // The glyph box is a 16px column at the control's full height,
              // so a landscape camera takes its width and a portrait one is
              // held to the large icon height, which keeps it inside the
              // column rather than squeezed out of shape by it.
              className={
                cameraStageSize !== null &&
                cameraStageSize.height > cameraStageSize.width
                  ? "h-icon-large"
                  : "w-full"
              }
              frameSize={cameraStageSize}
              hasFrame={cameraPreview.hasFrame}
              isFlipped={cameraFlipped}
            />
          </span>
        </ToggleMenuButton>
      </div>

      <div className="flex" data-popup-panel-trigger={MICROPHONE_PANEL_ID}>
        <ToggleMenuButton
          aria-label={t("recording-controls-microphone")}
          badge={inputBadge(
            inputs.microphone ? microphoneWarning : undefined,
            isMicrophoneLocked,
          )}
          detail={inputMeter(
            inputs.microphone,
            microphonePreview.decibels,
            microphonePreview.peak,
          )}
          isMenuDisabled={isDisabled}
          isSelected={inputs.microphone}
          isToggleDisabled={isDisabled}
          label={microphoneName(selectedMicrophone)}
          menuLabel={t("recording-controls-choose-microphone")}
          off={<MicOff />}
          onChange={(selected) => {
            void closeIfOpen(MICROPHONE_PANEL_ID);
            if (isMicrophoneLocked) onMicrophoneLockedPress?.();
            else onInputChange("microphone", selected);
          }}
          onMenuPress={(anchor, fromKeyboard) => {
            void showMicrophonePicker({
              anchor,
              focusContents: fromKeyboard,
              isMicrophoneOn: inputs.microphone,
              refreshMicrophones,
              selectedMicrophone,
            });
          }}
          renderToggle={warnedToggle(
            inputs.microphone ? microphoneWarning : undefined,
          )}
          size="capture"
          variant="ghost"
        >
          <Mic />
        </ToggleMenuButton>
      </div>

      <div className="flex" data-popup-panel-trigger={SYSTEM_AUDIO_PANEL_ID}>
        <ToggleMenuButton
          aria-label={t("recording-controls-system-audio-picker")}
          badge={inputBadge(
            inputs.systemAudio ? systemAudioWarning : undefined,
            false,
          )}
          detail={inputMeter(
            inputs.systemAudio,
            systemAudioPreview.decibels,
            systemAudioPreview.peak,
          )}
          isMenuDisabled={isDisabled}
          isSelected={inputs.systemAudio}
          isToggleDisabled={isDisabled}
          label={systemAudioName(selectedSystemAudio)}
          menuLabel={t("recording-controls-choose-system-audio")}
          off={<VolumeX />}
          onChange={(selected) => {
            void closeIfOpen(SYSTEM_AUDIO_PANEL_ID);
            onInputChange("systemAudio", selected);
          }}
          onMenuPress={(anchor, fromKeyboard) => {
            void showSystemAudioPicker({
              anchor,
              focusContents: fromKeyboard,
              refreshAudioSources,
              selectedSystemAudio,
            });
          }}
          renderToggle={warnedToggle(
            inputs.systemAudio ? systemAudioWarning : undefined,
          )}
          size="capture"
          variant="ghost"
        >
          <Volume2 />
        </ToggleMenuButton>
      </div>
    </div>
  );
}

export type { RecordingBarInputsProps };
