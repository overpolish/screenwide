// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getCurrentWindow } from "@tauri-apps/api/window";
import { createContext, use } from "react";

import * as api from "./api";

export const SettingsApiContext = createContext({
  ...api,
  minimize: () => getCurrentWindow().minimize(),
});

export const useSettingsApi = () => use(SettingsApiContext);
