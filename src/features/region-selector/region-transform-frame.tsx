// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";
import { Rnd } from "react-rnd";

import { TransformControls } from "../../components/shared/canvas-tools/transform-controls";
import { cn } from "../../lib/styling";
import { Region } from "../recording-sources/types";

import { HANDLE_CLASSES, HANDLE_STYLES } from "./resize-handles";
import { ResizeDirection } from "./types";

export function RegionTransformFrame({
  aspectRatio,
  freeAspect,
  onChange,
  onDraggingChange,
  onGestureBegin,
  onGestureFinish,
  onPersist,
  onResizeDirectionChange,
  region,
  visible,
}: {
  aspectRatio: number | false;
  freeAspect: boolean;
  onChange: (region: Region) => void;
  onDraggingChange: (dragging: boolean) => void;
  onGestureBegin: () => void;
  onGestureFinish: () => void;
  onPersist: () => void;
  onResizeDirectionChange: (direction: ResizeDirection | undefined) => void;
  region: Region;
  visible: boolean;
}) {
  const [resizeDirection, setResizeDirection] = useState<ResizeDirection>();
  const [freedThisResize, setFreedThisResize] = useState(false);

  useEffect(() => {
    // re-resizable fixes the ratio when a resize starts, so handing it a
    // number again mid-gesture snaps the region back to the shape it began
    // with. Freeing once therefore has to hold until the gesture ends.
    if (!freeAspect || !resizeDirection) return;
    // eslint-disable-next-line @eslint-react/set-state-in-effect
    setFreedThisResize(true);
  }, [freeAspect, resizeDirection]);

  const changeResizeDirection = (direction: ResizeDirection | undefined) => {
    setResizeDirection(direction);
    onResizeDirectionChange(direction);
  };

  return (
    <Rnd
      bounds="parent"
      className={cn(
        "relative transition-opacity",
        !visible && "invisible opacity-0",
      )}
      dragGrid={[1, 1]}
      lockAspectRatio={
        freeAspect || freedThisResize ? false : aspectRatio || false
      }
      onDrag={(_event, data) => {
        onChange({
          ...region,
          position: { x: data.x, y: data.y },
        });
      }}
      onDragStart={() => {
        onGestureBegin();
        onDraggingChange(true);
      }}
      onDragStop={() => {
        onPersist();
        onDraggingChange(false);
        onGestureFinish();
      }}
      // react-rnd defines this callback with five required parameters.
      // eslint-disable-next-line @typescript-eslint/max-params
      onResize={(_event, _direction, element, _delta, position) => {
        onChange({
          position,
          size: {
            height: Number.parseInt(element.style.height, 10),
            width: Number.parseInt(element.style.width, 10),
          },
        });
      }}
      onResizeStart={(_event, direction) => {
        onGestureBegin();
        changeResizeDirection(direction);
      }}
      onResizeStop={() => {
        onPersist();
        changeResizeDirection(undefined);
        setFreedThisResize(false);
        onGestureFinish();
      }}
      position={region.position}
      resizeGrid={[1, 1]}
      resizeHandleClasses={HANDLE_CLASSES}
      resizeHandleStyles={HANDLE_STYLES}
      size={region.size}
    >
      {/* The same marquee chrome as the export window's crop controls;
          react-rnd supplies behaviour through its own invisible handles. */}
      <TransformControls
        frame={{
          height: region.size.height,
          width: region.size.width,
          x: 0,
          y: 0,
        }}
        inverseScale="1"
      />
    </Rnd>
  );
}
