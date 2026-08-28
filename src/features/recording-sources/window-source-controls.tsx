// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { WandSparkles } from "lucide-react";
import { useState } from "react";

import { IconButton } from "../../components/base/button/icon-button";
import { CheckOnClick } from "../../components/shared/check-on-click/check-on-click";
import { Dimensions } from "../../components/shared/dimensions/dimensions";

import { resizeWindow } from "./api";
import { WindowDetails } from "./types";

type WindowSourceControlsProps = {
  selectedWindow: WindowDetails | null;
};

function SelectedWindowControls({ selectedWindow }: WindowSourceControlsProps) {
  const [width, setWidth] = useState(selectedWindow?.size.width ?? 1920);
  const [height, setHeight] = useState(selectedWindow?.size.height ?? 1080);

  return (
    <div className="gap-control flex items-center">
      <Dimensions
        height={height}
        setDimensions={(nextWidth, nextHeight) => {
          setWidth(nextWidth);
          setHeight(nextHeight);
        }}
        setHeight={setHeight}
        setWidth={setWidth}
        width={width}
      />
      <CheckOnClick
        blur="xs"
        onPress={() => {
          if (selectedWindow)
            return resizeWindow(selectedWindow, width, height);
        }}
      >
        <IconButton
          aria-label="Apply dimensions"
          isDisabled={!selectedWindow}
          size="compact"
        >
          <WandSparkles />
        </IconButton>
      </CheckOnClick>
    </div>
  );
}

export function WindowSourceControls(props: WindowSourceControlsProps) {
  const { selectedWindow } = props;
  return (
    <SelectedWindowControls
      key={
        selectedWindow
          ? `${String(selectedWindow.pid)}:${String(selectedWindow.id)}`
          : "none"
      }
      {...props}
    />
  );
}
