// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Checkbox } from "../../../components/base/checkbox/checkbox";
import { ColorSwatch } from "../../../components/base/input-fields/color-swatch";
import { PillGroup } from "../../../components/base/pill-group/pill-group";
import { ScrollArea } from "../../../components/base/scroll-area/scroll-area";
import { Dimensions } from "../../../components/shared/dimensions/dimensions";
import { ScreenshotOutputSettings } from "../screenshot-output";

import { MeshBackgroundControl } from "./mesh-background-control";

const backgroundTypes = [
  { id: "solid", label: "Solid" },
  { id: "mesh", label: "Mesh" },
];

export function ScreenshotInspector({
  isSaving,
  onChange,
  onDimensionsChange,
  settings,
  sourceHeight,
  sourceWidth,
}: {
  settings: ScreenshotOutputSettings;
  sourceHeight: number;
  sourceWidth: number;
  isSaving?: boolean;
  onChange?: (settings: ScreenshotOutputSettings) => void;
  onDimensionsChange?: (width: number, height: number) => void;
}) {
  return (
    <aside className="flex min-h-0 min-w-0 flex-col border-r border-muted/15 bg-content/35">
      <ScrollArea rootClassName="min-h-0 grow">
        <ScreenshotOutputControls
          isSaving={isSaving}
          onChange={onChange}
          onDimensionsChange={onDimensionsChange}
          settings={settings}
          sourceHeight={sourceHeight}
          sourceWidth={sourceWidth}
        />
      </ScrollArea>
    </aside>
  );
}

export function ScreenshotOutputControls({
  className = "p-4",
  isSaving,
  onChange,
  onDimensionsChange,
  settings,
  showDropShadow = true,
  sourceHeight,
  sourceWidth,
}: {
  settings: ScreenshotOutputSettings;
  sourceHeight: number;
  sourceWidth: number;
  className?: string;
  isSaving?: boolean;
  onChange?: (settings: ScreenshotOutputSettings) => void;
  onDimensionsChange?: (width: number, height: number) => void;
  showDropShadow?: boolean;
}) {
  const update = (patch: Partial<ScreenshotOutputSettings>) => {
    onChange?.({ ...settings, ...patch });
  };
  const resizeCanvas = (width: number, height: number) => {
    if (onDimensionsChange) {
      onDimensionsChange(Math.round(width), Math.round(height));
      return;
    }
    onChange?.({
      ...settings,
      height: Math.round(height),
      width: Math.round(width),
    });
  };

  return (
    <div className={`flex flex-col gap-4 ${className}`}>
      <div className={isSaving ? "pointer-events-none opacity-50" : ""}>
        <Dimensions
          height={settings.height}
          initialLinked
          layout="stacked"
          onReset={
            settings.width !== sourceWidth || settings.height !== sourceHeight
              ? () => {
                  resizeCanvas(sourceWidth, sourceHeight);
                }
              : undefined
          }
          setDimensions={(width, height) => {
            resizeCanvas(width, height);
          }}
          setHeight={(height) => {
            resizeCanvas(settings.width, height);
          }}
          setWidth={(width) => {
            resizeCanvas(width, settings.height);
          }}
          width={settings.width}
        />
      </div>

      <div className="flex items-center justify-between gap-3">
        <span className="text-xs text-content-fg">Background</span>
        <PillGroup
          aria-label="Screenshot background type"
          display="label"
          isDisabled={isSaving}
          items={backgroundTypes}
          onSelectionChange={(backgroundType) => {
            update({
              backgroundType:
                backgroundType as ScreenshotOutputSettings["backgroundType"],
            });
          }}
          selected={settings.backgroundType}
        />
      </div>

      {settings.backgroundType === "solid" ? (
        <div className="flex items-center justify-between gap-3">
          <span className="text-xs text-content-fg">Colour</span>
          <ColorSwatch
            ariaLabel="Background colour"
            isDisabled={isSaving}
            onChange={(backgroundColor) => {
              update({ backgroundColor });
            }}
            value={settings.backgroundColor}
          />
        </div>
      ) : (
        <MeshBackgroundControl
          isDisabled={isSaving}
          onChange={onChange}
          settings={settings}
        />
      )}

      {showDropShadow ? (
        <Checkbox
          isDisabled={isSaving}
          isSelected={settings.dropShadow}
          onChange={(dropShadow) => {
            update({ dropShadow });
          }}
        >
          <span className="text-xs">Drop shadow</span>
        </Checkbox>
      ) : null}
    </div>
  );
}
