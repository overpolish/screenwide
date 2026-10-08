<!-- SPDX-FileCopyrightText: 2026 overpolish -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<div align="center">
	<img height="75px" alt="Screenwide logo" src="./src-tauri/icons/ScreenwideLogo.png" />
</div>

<h1 align="center">Screenwide</h1>

<div align="center">
	<img alt="Latest GitHub release" src="https://img.shields.io/github/v/release/overpolish/screenwide?display_name=tag&label=release&labelColor=5C5C5C&color=FF2970" />
	<img alt="Platforms: macOS and Windows" src="https://img.shields.io/badge/platforms-macOS_%7C_Windows-FF2970?logo=tauri&labelColor=5C5C5C&logoColor=white" />
	<img alt="GPL-3.0-or-later license" src="https://img.shields.io/github/license/overpolish/screenwide?labelColor=5C5C5C&color=FF2970" />
</div>

<br />

<div align="center">
	Everything screen.
</div>

<br />

<!--
<div align="center">
	<img width="18%" alt="Recording with Screenwide" src="./Assets/Marketing/screenwide-recording.png" />
	<img width="18%" alt="Editing a recording in Screenwide" src="./Assets/Marketing/screenwide-editor.png" />
	<img width="18%" alt="Taking a screenshot with Screenwide" src="./Assets/Marketing/screenwide-screenshots.png" />
	<img width="18%" alt="Measuring an interface with Screenwide Ruler" src="./Assets/Marketing/screenwide-ruler.png" />
	<img width="18%" alt="Recognizing text with Screenwide" src="./Assets/Marketing/screenwide-text-recognition.png" />
</div>

<br />
-->

# Core features

- **Screen recording.** Record all of the screen or just the part you need. Camera and audio are optional.
- **Screenshots.** Capture a selected area or a scrolling page. Combine several captures if needed.
- **Glide.** Move and resize windows with a fluid trackpad or mouse gesture. Snap them into halves, quarters, thirds or full screen-or glide down to minimize.
- **Better cursors.** Make the cursor easier to follow, with control over its size and movement.
- **Keyboard shortcuts.** Show pressed shortcuts on the recording.
- **Ruler.** Measure anything on screen and copy the result.
- **Text recognition.** Copy text from anywhere on screen. QR codes work too.

<br />

# Roadmap

<div>
	<h3>Wallpapers and Custom Backgrounds</h3>
	Your current desktop wallpaper and custom images.
</div>

<div>
	<h3>AI Transcription</h3>
	Turn recorded speech into a timed, editable transcript.
</div>

<div>
	<h3>AI Captions</h3>
	Generate synchronized captions directly from a recording.
</div>

<div>
	<h3>Text-based Editing</h3>
	Edit a recording by editing its transcript.
</div>

<div>
	<h3>Live Transcription</h3>
	Transcribe live without keeping an audio recording.
</div>

<div>
	<h3>AI Auto Zooms</h3>
	Intelligent zooms to areas of interest.
</div>

<div>
	<h3>Custom Glide Grids</h3>
	Create custom Glide grid layouts beyond 2×2 and 3×3.
</div>

<div>
	<h3>Shader Filters</h3>
	GLSL shader support for filters (CRT)/custom backgrounds.
</div>

<br />

# Building on Windows

Besides Rust, Node.js, pnpm and the Visual Studio C++ Build Tools, the speech-to-text helper (`pnpm transcriber:prepare`, run by `pnpm dev` and `pnpm build`) compiles whisper.cpp, which needs LLVM for its bindings, CMake, and the Vulkan SDK for its GPU shaders. The same step downloads the voice activity model the microphone tools use, once, and checks it against a pinned digest. Tauri checks that the helper and the model exist before compiling the app, so even `cargo check` fails until they have been prepared once. Use LLVM 20: with LLVM 23 the bindgen version that whisper-rs uses leaves `whisper_full_params` opaque, and the build fails a layout check.

```powershell
winget install --id LLVM.LLVM -e --version 20.1.8
winget install Kitware.CMake KhronosGroup.VulkanSDK
setx LIBCLANG_PATH "C:\Program Files\LLVM\bin"
```

Then fully restart the terminal app (Windows Terminal, VS Code, and so on) so `VULKAN_SDK`, `LIBCLANG_PATH` and the CMake path are picked up. A new tab or pane inherits the environment the app started with. Then run `pnpm transcriber:prepare`.
