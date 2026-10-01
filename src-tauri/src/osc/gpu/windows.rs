// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows OSC geometry, shader contract, wgpu pipeline and portable palettes.

mod pipeline;
mod renderer;

pub(crate) use pipeline::{bind_group_layout, pipeline, sampler, shader_module};
pub(crate) use renderer::*;
