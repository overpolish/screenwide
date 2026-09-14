// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { withThemeByClassName } from "@storybook/addon-themes";
import { mockIPC } from "@tauri-apps/api/mocks";
import { themes } from "storybook/theming";

import { installKeyboardNavigationModality } from "../src/lib/keyboard-navigation-modality";
import { synchronizeSystemAccent } from "../src/lib/system-accent";

import type { Decorator, Preview } from "@storybook/react-vite";

import "../src/index.css";
import "./styles.css";

const isNativePreview =
  new URLSearchParams(window.location.search).get("screenwide-native") === "1";

// The skin is a token layer keyed off `data-platform`, as the app sets it in
// `main.tsx`. Stories default to the host OS so a reviewer sees their own
// platform first, and the toolbar switches to the other one. The native
// preview runs inside the real app shell, so it always takes the host.
type Platform = "macos" | "windows";
const hostPlatform: Platform = navigator.userAgent.includes("Windows")
  ? "windows"
  : "macos";

const applyPlatform = (platform: Platform) => {
  if (platform === "windows") {
    document.documentElement.dataset.platform = "windows";
  } else {
    delete document.documentElement.dataset.platform;
  }
};

// The theme decorator applies its class only once the manager has sent the
// globals, so every reload rendered light first and then flipped. The iframe
// URL already carries the chosen theme in `globals`, so the class is applied
// here, before any story renders; the decorator takes over from there.
if (!isNativePreview) {
  const globals = new URLSearchParams(window.location.search).get("globals");
  const theme = /(?:^|;)theme:(\w+)/.exec(globals ?? "")?.[1] ?? "dark";
  document.documentElement.classList.add(theme === "light" ? "light" : "dark");
  const platform = /(?:^|;)platform:(\w+)/.exec(globals ?? "")?.[1];
  applyPlatform(
    platform === "windows" || platform === "macos" ? platform : hostPlatform,
  );
}

if (isNativePreview) {
  document.documentElement.classList.add("screenwide-native-preview");
  const systemDarkMode = window.matchMedia("(prefers-color-scheme: dark)");
  const synchronizeNativeTheme = ({
    matches,
  }: MediaQueryListEvent | MediaQueryList) => {
    document.documentElement.classList.toggle("dark", matches);
    document.documentElement.classList.toggle("light", !matches);
  };

  synchronizeNativeTheme(systemDarkMode);
  systemDarkMode.addEventListener("change", synchronizeNativeTheme);
  applyPlatform(hostPlatform);
  // The native preview runs inside the app shell, so the OS accent is real.
  synchronizeSystemAccent();
}

// Stub the Tauri runtime so stories mounting Tauri-touching components render
// outside the desktop app. This installs `window.__TAURI_INTERNALS__` with
// `invoke`, `transformCallback`, `unregisterCallback`, `runCallback`, and the
// event-plugin handler. Without it, e.g. `new Channel()` throws because
// `transformCallback` is undefined. Run as a module-load side effect so the
// runtime exists before the first story's effects fire. Data queries must keep
// their response shape, even without native content. Unknown commands resolve
// to `null`; these stories render layout rather than live pixels.
if (!isNativePreview) {
  mockIPC(
    (command) => {
      if (command === "get_recording_keyboard_timeline") return [];
      return null;
    },
    { shouldMockEvents: true },
  );
}

// Initialize React Aria's focus tracking before Storybook's test loader wraps
// HTMLElement.prototype.focus. Lazy initialization after that wrapper is unsafe.
const [reactAria, storybookComponents] = await Promise.all([
  import("react-aria"),
  import("storybook/internal/components"),
]);

Object.freeze([reactAria.useOverlay, storybookComponents.Button]);

const disposeKeyboardNavigation = installKeyboardNavigationModality();
const previewHot = (
  import.meta as { hot?: { dispose: (cb: () => void) => void } }
).hot;
previewHot?.dispose(disposeKeyboardNavigation);

const preview: Preview = {
  globalTypes: isNativePreview
    ? {}
    : {
        platform: {
          description: "Platform skin",
          toolbar: {
            dynamicTitle: true,
            icon: hostPlatform === "windows" ? "windows" : "apple",
            items: [
              { icon: "apple", title: "macOS", value: "macos" },
              { icon: "windows", title: "Windows", value: "windows" },
            ],
            title: "Platform",
          },
        },
      },
  initialGlobals: isNativePreview ? {} : { platform: hostPlatform },
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
    docs: {
      theme: themes.dark,
    },
    options: {
      storySort: {
        method: "alphabetical",
      },
    },
  },
  tags: ["autodocs"],
};

const withPlatform: Decorator = (Story, { globals }) => {
  applyPlatform(globals.platform === "windows" ? "windows" : "macos");
  return Story();
};

export const decorators = (
  isNativePreview
    ? []
    : [
        withPlatform,
        withThemeByClassName({
          defaultTheme: "dark",
          parentSelector: "html",
          themes: {
            dark: "dark",
            light: "light",
          },
        }),
      ]
) as Decorator[];

export default preview;
