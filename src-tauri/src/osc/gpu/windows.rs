// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows OSC geometry: the vertex builders the shared pipeline draws.

mod renderer;

pub(crate) use super::pipeline::{
  bind_group_layout, pipeline, sampler, shader_module, RenderConstants, Vertex,
};
pub(crate) use renderer::*;
