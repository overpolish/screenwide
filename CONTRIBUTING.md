<!-- SPDX-FileCopyrightText: 2026 overpolish -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Contributing to Screenwide

Screenwide is a Tauri app: the interface is React and TypeScript, and capture, drawing and everything else native is Rust. How the code is laid out, named and written, and how the interface is put together, is in [AGENTS.md](AGENTS.md); read it before changing code.

## Set up

Every platform needs [Rust](https://rustup.rs), [Node.js](https://nodejs.org) and pnpm. The pnpm version is pinned in `package.json` (`packageManager`), so `corepack enable` picks the right one.

The first `pnpm dev` (or `pnpm build`) prepares what the app ships beside itself: FFmpeg, the speech-to-text helper and its voice activity model, and on Windows the shader compiler and the window hider. Each is prepared once and checked against pinned digests after that. Tauri checks they exist before compiling the app, so even `cargo check` fails until they have been prepared once.

### macOS

Screenwide runs on Apple Silicon only. Install Xcode's command line tools, then CMake for the speech-to-text helper and pkg-config for the FFmpeg build:

```sh
xcode-select --install
brew install cmake pkg-config
```

FFmpeg and x264 are compiled from pinned sources the first time (`scripts/build-ffmpeg-macos.sh`, see [FFMPEG.md](FFMPEG.md)), which takes a few minutes.

### Windows

Install the Visual Studio C++ Build Tools with the C++ desktop workload. The speech-to-text helper compiles whisper.cpp, which also needs:

- LLVM 20, for its bindings. With LLVM 23 the bindgen version whisper-rs uses leaves `whisper_full_params` opaque, and the build fails a layout check.
- CMake 4.2 or later.
- Ninja. The Build Tools' C++ CMake tools for Windows component, part of the C++ desktop workload by default, includes it; the build finds it there when `ninja` is not on PATH.
- The Vulkan SDK, for its GPU shaders.

```powershell
winget install --id LLVM.LLVM -e --version 20.1.8
winget install Kitware.CMake KhronosGroup.VulkanSDK
setx LIBCLANG_PATH "C:\Program Files\LLVM\bin"
```

Then fully restart the terminal app (Windows Terminal, VS Code and so on) so `VULKAN_SDK`, `LIBCLANG_PATH` and the CMake path are picked up: a new tab or pane inherits the environment the app started with. Then run `pnpm transcriber:prepare`.

Building the helper also downloads a prebuilt ONNX Runtime with DirectML, which runs the Studio sound and Studio ducking models on the GPU, links it in, and puts `DirectML.dll` beside the helper.

Windows Defender scans every file a build writes, which can make Rust builds several times slower. Build from a [Dev Drive](https://learn.microsoft.com/windows/dev-drive/), whose performance mode defers those scans, or exclude the repository's `src-tauri\target`, `%USERPROFILE%\.cargo` and `%USERPROFILE%\.rustup` folders from real-time protection.

## Run

- `pnpm tauri dev` runs the app, rebuilding as the code changes.
- `pnpm storybook` shows every primitive, composite and feature on its own. `pnpm dev:storybook-native` shows them in the app's real native windows, which menus, popovers and tooltips open in.
- `pnpm dev:recording-failures`, `pnpm dev:screenshot-failures` and `pnpm dev:permissions` run the app with failures or missing permissions faked, to see how each is shown.
- `pnpm site:dev` serves the website in `src/site`, built from the app's own components; `pnpm site:build` builds it into `dist-site`, and `pnpm site:deploy` builds it and publishes it to getscreenwide.com through the static Cloudflare Worker in `workers/site`.

## Check

`pnpm check` runs everything the repository's continuous integration runs, on macOS and Windows: licence headers, generated bindings, translations, source sizes, Tailwind classes, unused code, formatting, the TypeScript and Rust tests, both builds and Clippy. While working, run the parts that cover your change; [AGENTS.md](AGENTS.md) lists them.

## Licences

Most of Screenwide is under the GPL; the paid tools and their licence check are under the Elastic License 2.0. `scripts/check-spdx.mjs` lists the paths each licence covers, and `pnpm license:check` makes sure every file's SPDX header names the right one. A new file in a tool's folder takes `SPDX-License-Identifier: Elastic-2.0`; any other file takes `GPL-3.0-or-later`.

## Also

- [TRANSLATING.md](TRANSLATING.md): adding a language and translating the interface.
- [RELEASING.md](RELEASING.md): making a release and testing updates.
- [FFMPEG.md](FFMPEG.md): how FFmpeg is built and distributed.

Commit messages follow [Conventional Commits](https://www.conventionalcommits.org): `type(scope): summary`.
