// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Activity,
  Camera,
  CameraOff,
  FlipHorizontal2,
  Lock,
  Mic,
  Scan,
  Volume2,
} from "lucide-react";
import { RefObject } from "react";
import { TooltipTrigger } from "react-aria-components";

import { Button } from "../../components/base/button/button";
import { IconToggleButton } from "../../components/base/button/icon-button";
import { CircularProgress } from "../../components/base/circular-progress/circular-progress";
import {
  FieldGroup,
  FieldGroupAction,
  FieldGroupFooter,
} from "../../components/base/field-group/field-group";
import { ListBoxItem } from "../../components/base/listbox-item/listbox-item";
import { Select } from "../../components/base/select/select";
import { Tooltip } from "../../components/base/tooltip/tooltip";
import { cn } from "../../lib/styling";
import { AudioMeter } from "../audio-inputs/components/audio-meter";
import { StandaloneMultiSelect } from "../standalone-listbox/standalone-multi-select";
import { StandaloneSelect } from "../standalone-listbox/standalone-select";

import { cameraPreviewFitClassName } from "./camera-preview-fit";
import {
  CameraDevice,
  CameraResolution,
  InputDevice,
  SystemAudioSource,
} from "./types";

type InputSelectProps<T extends InputDevice> = {
  icon: React.ReactNode;
  id: string;
  items: T[];
  label: string;
  onChange: (item: T) => void;
  placeholder: string;
  selected: T | null;
  standalone: boolean;
  onOpen?: () => Promise<T[]>;
};

function InputSelect<T extends InputDevice>({
  icon,
  id,
  items,
  label,
  onChange,
  onOpen,
  placeholder,
  selected,
  standalone,
}: InputSelectProps<T>) {
  if (standalone) {
    return (
      <StandaloneSelect
        id={id}
        items={items}
        label={label}
        leftSection={icon}
        onOpen={onOpen}
        onSelectionChange={(item) => {
          const match = items.find((candidate) => candidate.id === item.id);
          if (match) onChange(match);
        }}
        placeholder={placeholder}
        selectedId={selected?.id ?? null}
      />
    );
  }

  return (
    <Select<T>
      aria-label={label}
      className="w-full"
      clearable={false}
      items={items}
      leftSection={icon}
      onChange={(selection) => {
        const match = items.find((item) => item.id === selection);
        if (match) onChange(match);
      }}
      placeholder={placeholder}
      size="compact"
      value={selected?.id ?? null}
    >
      {(item: T) => (
        <ListBoxItem id={item.id} textValue={item.label}>
          {item.label}
        </ListBoxItem>
      )}
    </Select>
  );
}

type SystemAudioSelectProps = {
  items: SystemAudioSource[];
  onChange: (items: SystemAudioSource[]) => void;
  selected: SystemAudioSource[];
  standalone: boolean;
  onOpen?: () => Promise<SystemAudioSource[]>;
};

function SystemAudioSelect({
  items,
  onChange,
  onOpen,
  selected,
  standalone,
}: SystemAudioSelectProps) {
  if (standalone) {
    return (
      <StandaloneMultiSelect
        exclusiveId="all"
        id="system-audio"
        items={items}
        label="System audio"
        leftSection={<Volume2 className="size-icon-compact" />}
        onOpen={onOpen}
        onSelectionChange={(selection) => {
          onChange(
            selection
              .map((item) =>
                items.find((candidate) => candidate.id === item.id),
              )
              .filter((item): item is SystemAudioSource => item !== undefined),
          );
        }}
        placeholder="No system audio"
        selectedIds={selected.map((item) => item.id)}
      />
    );
  }

  return (
    <Select<SystemAudioSource>
      aria-label="System audio"
      className="w-full"
      clearable={false}
      items={items}
      leftSection={<Volume2 className="size-icon-compact" />}
      onChange={(selection) => {
        const match = items.find((item) => item.id === selection);
        if (match) onChange([match]);
      }}
      placeholder="No system audio"
      size="compact"
      value={selected[0]?.id ?? null}
    >
      {(item: SystemAudioSource) => (
        <ListBoxItem id={item.id} textValue={item.label}>
          {item.label}
        </ListBoxItem>
      )}
    </Select>
  );
}

type PermissionOverlayProps = {
  label: string;
  onPress?: () => void;
};

function PermissionOverlay({ label, onPress }: PermissionOverlayProps) {
  return (
    <div className="absolute inset-0 z-10 flex items-center justify-center rounded-xl bg-content/80 backdrop-blur-sm">
      <Button
        className="gap-control"
        onPress={onPress}
        size="compact"
        variant="ghost"
      >
        <Lock className="size-icon-compact" />
        {label}
      </Button>
    </div>
  );
}

export type RecordingOptionsProps = {
  audioSources: SystemAudioSource[];
  cameras: CameraDevice[];
  microphones: InputDevice[];
  onCameraChange: (camera: CameraDevice) => void;
  onCameraResolutionChange: (resolution: CameraResolution) => void;
  onMicrophoneChange: (microphone: InputDevice) => void;
  onSystemAudioChange: (sources: SystemAudioSource[]) => void;
  selectedCamera: CameraDevice | null;
  selectedCameraResolution: CameraResolution | null;
  selectedMicrophone: InputDevice | null;
  selectedSystemAudio: SystemAudioSource[];
  cameraFlipped?: boolean;
  cameraLocked?: boolean;
  cameraPal?: boolean;
  cameraPreviewActive?: boolean;
  cameraPreviewRef?: RefObject<HTMLCanvasElement | null>;
  /** The preview is wanted but has not drawn a frame yet. */
  cameraPreviewStarting?: boolean;
  microphoneDecibels?: number;
  microphoneLocked?: boolean;
  microphonePeak?: number;
  microphonePreviewEnabled?: boolean;
  onCameraFlippedChange?: (flipped: boolean) => void;
  onCameraLockedPress?: () => void;
  onCameraOptionsOpen?: () => Promise<CameraDevice[]>;
  onCameraPalChange?: (pal: boolean) => void;
  onMicrophoneLockedPress?: () => void;
  onMicrophoneOptionsOpen?: () => Promise<InputDevice[]>;
  onSystemAudioOptionsOpen?: () => Promise<SystemAudioSource[]>;
  standalone?: boolean;
  systemAudioDecibels?: number;
  systemAudioPeak?: number;
  systemAudioPreviewEnabled?: boolean;
};

export function RecordingOptions({
  audioSources,
  cameraFlipped = false,
  cameraLocked = false,
  cameraPal = false,
  cameraPreviewActive = false,
  cameraPreviewRef,
  cameraPreviewStarting = false,
  cameras,
  microphoneDecibels = -Infinity,
  microphoneLocked = false,
  microphonePeak = -Infinity,
  microphonePreviewEnabled = false,
  microphones,
  onCameraChange,
  onCameraFlippedChange,
  onCameraLockedPress,
  onCameraOptionsOpen,
  onCameraPalChange,
  onCameraResolutionChange,
  onMicrophoneChange,
  onMicrophoneLockedPress,
  onMicrophoneOptionsOpen,
  onSystemAudioChange,
  onSystemAudioOptionsOpen,
  selectedCamera,
  selectedCameraResolution,
  selectedMicrophone,
  selectedSystemAudio,
  standalone = false,
  systemAudioDecibels = -Infinity,
  systemAudioPeak = -Infinity,
  systemAudioPreviewEnabled = false,
}: RecordingOptionsProps) {
  return (
    <main className="window-surface gap-section p-section flex w-full min-w-[240px] flex-col overflow-hidden text-content-fg">
      <section className="gap-section relative flex flex-col">
        {cameraLocked ? (
          <PermissionOverlay
            label="Grant camera access"
            onPress={onCameraLockedPress}
          />
        ) : null}

        <div className="relative flex aspect-video w-full shrink-0 items-center justify-center text-muted">
          <div className="inset-control absolute flex min-h-0 min-w-0 items-center justify-center">
            <canvas
              aria-label="Camera preview"
              className={cn(
                "shadow-preview block shrink-0 self-center",
                selectedCameraResolution &&
                  cameraPreviewFitClassName(selectedCameraResolution),
                cameraFlipped && "-scale-x-100",
              )}
              hidden={!cameraPreviewActive}
              ref={cameraPreviewRef}
              role="img"
            />
          </div>
          {!cameraPreviewActive ? (
            cameraPreviewStarting ? (
              <CircularProgress
                aria-label="Starting camera preview"
                isIndeterminate
              />
            ) : (
              <CameraOff className="size-icon-prominent" />
            )
          ) : null}
        </div>

        <FieldGroup className="flex items-center">
          <div className="min-w-0 flex-1">
            <InputSelect
              icon={<Camera className="size-icon-compact" />}
              id="camera"
              items={cameras}
              label="Camera"
              onChange={onCameraChange}
              onOpen={onCameraOptionsOpen}
              placeholder="No camera"
              selected={selectedCamera}
              standalone={standalone}
            />
          </div>
          {selectedCamera && onCameraFlippedChange ? (
            <FieldGroupAction>
              <TooltipTrigger delay={400}>
                <IconToggleButton
                  aria-label="Flip camera horizontally"
                  isSelected={cameraFlipped}
                  onChange={onCameraFlippedChange}
                  size="compact"
                >
                  <FlipHorizontal2 />
                </IconToggleButton>
                <Tooltip placement="top">Flip camera</Tooltip>
              </TooltipTrigger>
            </FieldGroupAction>
          ) : null}
        </FieldGroup>
        <FieldGroup className="flex items-center">
          <div className="min-w-0 flex-1">
            <InputSelect
              icon={<Scan className="size-icon-compact" />}
              id="camera-resolution"
              items={selectedCamera?.modes ?? []}
              label="Camera resolution"
              onChange={onCameraResolutionChange}
              placeholder="No resolution"
              selected={selectedCameraResolution}
              standalone={standalone}
            />
          </div>
          {selectedCamera && onCameraPalChange ? (
            <FieldGroupAction>
              <TooltipTrigger delay={400}>
                <IconToggleButton
                  aria-label="Anti-flicker"
                  isSelected={cameraPal}
                  onChange={onCameraPalChange}
                  size="compact"
                >
                  <Activity className="transform-gpu" />
                </IconToggleButton>
                <Tooltip placement="top">Anti-flicker</Tooltip>
              </TooltipTrigger>
            </FieldGroupAction>
          ) : null}
        </FieldGroup>
      </section>

      <section className="relative">
        {microphoneLocked ? (
          <PermissionOverlay
            label="Grant microphone access"
            onPress={onMicrophoneLockedPress}
          />
        ) : null}

        <FieldGroup className="gap-control flex flex-col">
          <InputSelect
            icon={<Mic className="size-icon-compact" />}
            id="microphone"
            items={microphones}
            label="Microphone"
            onChange={onMicrophoneChange}
            onOpen={onMicrophoneOptionsOpen}
            placeholder="No microphone"
            selected={selectedMicrophone}
            standalone={standalone}
          />
          <FieldGroupFooter>
            <AudioMeter
              decibels={microphoneDecibels}
              disabled={!microphonePreviewEnabled}
              height={5}
              hidePeakTick
              hideTicks
              peak={microphonePeak}
              width="100%"
            />
          </FieldGroupFooter>
        </FieldGroup>
      </section>

      <section>
        <FieldGroup className="gap-control flex flex-col">
          <SystemAudioSelect
            items={audioSources}
            onChange={onSystemAudioChange}
            onOpen={onSystemAudioOptionsOpen}
            selected={selectedSystemAudio}
            standalone={standalone}
          />
          <FieldGroupFooter>
            <AudioMeter
              decibels={systemAudioDecibels}
              disabled={!systemAudioPreviewEnabled}
              height={5}
              hidePeakTick
              hideTicks
              peak={systemAudioPeak}
              width="100%"
            />
          </FieldGroupFooter>
        </FieldGroup>
      </section>
    </main>
  );
}
