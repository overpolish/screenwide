// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  MenuHeading,
  MenuItem,
  MenuSeparator,
  MenuStandIn,
} from "../shared/recorder-parts";

// Everything set once and rarely touched, in one native menu beside Record:
// the countdown, what the recording shows of the pointer and keyboard, how
// screenshots are kept, and the camera's own settings.

export function PreflightOptions() {
  return (
    <MenuStandIn>
      <MenuHeading>Countdown</MenuHeading>
      <MenuItem isChecked>None</MenuItem>
      <MenuItem>3 seconds</MenuItem>
      <MenuItem>10 seconds</MenuItem>
      <MenuSeparator />
      <MenuItem isChecked>Show clicks</MenuItem>
      <MenuItem>Show keys pressed</MenuItem>
      <MenuSeparator />
      <MenuHeading>Screenshots</MenuHeading>
      <MenuItem>Copy instead of saving</MenuItem>
      <MenuItem>Capture scrolling region</MenuItem>
      <MenuSeparator />
      <MenuHeading>Camera</MenuHeading>
      <MenuItem>Flip horizontally</MenuItem>
      <MenuItem>Anti-flicker 50 Hz</MenuItem>
    </MenuStandIn>
  );
}
