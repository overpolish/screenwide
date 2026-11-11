// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A shader's layout checked against its WGSL before anything is compiled
//! against it: compiled code reads resources from the slots its layout gives
//! them, and a layout that left one out or gave it the wrong kind would draw
//! with whatever sits there instead of failing.

use naga::{AddressSpace, ImageClass, StorageAccess, TypeInner};
use screenwide_shader_layout::Binding;

use super::{entry_point, stages_of, Precompiled};

pub(super) fn layout(shader: &Precompiled, module: &naga::Module, info: &naga::valid::ModuleInfo) {
  let name = shader.name;
  for (group, entries) in shader.layout.groups.iter().enumerate() {
    assert!(
      entries
        .windows(2)
        .all(|pair| pair[0].binding < pair[1].binding),
      "the {name} shader's group {group} is not in ascending binding order, which wgpu numbers \
       it in"
    );
  }
  for &name_of_entry in shader.entry_points {
    let (index, entry) = entry_point(shader, module, name_of_entry);
    let entry_point = name_of_entry;
    let stage = stages_of(entry.stage);
    let uses = info.get_entry_point(index);
    for (handle, global) in module.global_variables.iter() {
      let Some(binding) = &global.binding else {
        continue;
      };
      if uses[handle].is_empty() {
        continue;
      }
      let what = global.name.as_deref().unwrap_or("a resource");
      let listed = shader
        .layout
        .groups
        .get(binding.group as usize)
        .and_then(|entries| {
          entries
            .iter()
            .find(|listed| listed.binding == binding.binding)
        })
        .unwrap_or_else(|| {
          panic!(
            "{entry_point} in the {name} shader uses {what} at group {} binding {}, which its \
             layout leaves out",
            binding.group, binding.binding
          )
        });
      assert!(
        listed.stages.contains(stage),
        "{entry_point} in the {name} shader uses {what}, which its layout keeps from that stage"
      );
      assert!(
        declared_as(module, global, listed.kind),
        "the {name} shader declares {what} as other than its layout's {:?}",
        listed.kind
      );
    }
  }
}

/// Whether `global` is declared as a resource of `kind`.
fn declared_as(module: &naga::Module, global: &naga::GlobalVariable, kind: Binding) -> bool {
  match (global.space, &module.types[global.ty].inner) {
    (AddressSpace::Uniform, _) => matches!(kind, Binding::Uniform | Binding::DynamicUniform),
    (AddressSpace::Storage { access }, _) => {
      kind
        == Binding::Storage {
          writable: access.contains(StorageAccess::STORE),
        }
    }
    (
      AddressSpace::Handle,
      TypeInner::Image {
        class: ImageClass::Storage { .. },
        ..
      },
    ) => matches!(kind, Binding::StorageTexture { .. }),
    (AddressSpace::Handle, TypeInner::Image { arrayed, .. }) => {
      matches!(kind, Binding::Texture { array, .. } if array == *arrayed)
    }
    (AddressSpace::Handle, TypeInner::Sampler { .. }) => matches!(kind, Binding::Sampler { .. }),
    _ => false,
  }
}
