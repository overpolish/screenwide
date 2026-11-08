// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! HT Demucs on DirectML, in parts.
//!
//! DirectML fuses a model into one graph, and HT Demucs whole is too much
//! for one: the full model runs out of memory every time, and the model
//! without its last step runs but returns nonsense, though each of its
//! operators gives the right answer in a smaller graph. Unfused, DirectML
//! holds the GPU long enough for Windows to reset it. So the model is cut
//! where its stages meet, each stage fused on its own: the encoders, the
//! cross-transformer and the decoders run on the GPU, and the inverse STFT,
//! which hangs the GPU in any form, on the CPU. ONNX keeps its nodes in an
//! order where each comes after what it reads, so any cut leaves a part
//! that runs on its own, given what earlier parts made.

use std::collections::{HashMap, HashSet};

use ort::session::{Session, SessionInputValue};
use ort::value::DynValue;
use prost::Message;
use tract_onnx::pb::{type_proto, GraphProto, ModelProto, TypeProto, ValueInfoProto};

use crate::session::{cpu_from_memory, directml_from_memory};

/// Where each part after the first begins: at the first node named with one
/// of these.
const STAGES: [&[&str]; 3] = [
  &["/crosstransformer"],
  &["/decoder.", "/tdecoder."],
  &["/real_istft"],
];

struct Part {
  session: Session,
  inputs: Vec<String>,
  outputs: Vec<String>,
}

pub(super) struct Staged {
  parts: Vec<Part>,
}

impl Staged {
  /// The model in `bytes`, cut into its stages, every stage but the last
  /// on the GPU.
  pub(super) fn new(bytes: &[u8]) -> Result<Self, String> {
    let model = ModelProto::decode(bytes).map_err(|error| error.to_string())?;
    let graph = model.graph.as_ref().ok_or("The model has no graph")?;
    let starts = stage_starts(graph)?;
    let types = element_types(graph);
    let count = starts.len() - 1;
    let parts = starts
      .windows(2)
      .enumerate()
      .map(|(index, bounds)| {
        let (bytes, inputs, outputs) = part(&model, graph, &types, bounds[0], bounds[1]);
        let session = if index + 1 < count {
          directml_from_memory(&bytes)
        } else {
          cpu_from_memory(&bytes)
        }
        .map_err(|error| format!("part {index} could not load: {error}"))?;
        Ok(Part {
          session,
          inputs,
          outputs,
        })
      })
      .collect::<Result<_, String>>()?;
    Ok(Self { parts })
  }

  /// The model's outputs for `mix`, by name.
  pub(super) fn run(&mut self, mix: DynValue) -> Result<HashMap<String, DynValue>, String> {
    let mut values = HashMap::from([("mix".to_owned(), mix)]);
    for (index, part) in self.parts.iter_mut().enumerate() {
      let inputs = part
        .inputs
        .iter()
        .map(|name| {
          let value = values
            .get(name)
            .ok_or_else(|| format!("nothing made {name}"))?;
          Ok((name.as_str(), SessionInputValue::from(value)))
        })
        .collect::<Result<Vec<_>, String>>()?;
      let mut outputs = part
        .session
        .run(inputs)
        .map_err(|error| format!("part {index} failed: {error}"))?;
      let made = part
        .outputs
        .iter()
        .map(|name| {
          let value = outputs
            .remove(name)
            .ok_or_else(|| format!("part {index} made no {name}"))?;
          Ok((name.clone(), value))
        })
        .collect::<Result<Vec<_>, String>>()?;
      drop(outputs);
      values.extend(made);
    }
    Ok(values)
  }
}

/// Where each part begins and where the last ends, as node indices.
fn stage_starts(graph: &GraphProto) -> Result<Vec<usize>, String> {
  let mut starts = vec![0];
  for prefixes in STAGES {
    let from = starts[starts.len() - 1];
    let start = graph.node[from..]
      .iter()
      .position(|node| prefixes.iter().any(|prefix| node.name.starts_with(prefix)))
      .map(|offset| from + offset)
      .filter(|&start| start > from)
      .ok_or_else(|| format!("the model has no stage named {}", prefixes[0]))?;
    starts.push(start);
  }
  starts.push(graph.node.len());
  Ok(starts)
}

/// The nodes from `from` up to `to` as a model of their own, with the
/// tensors they read from earlier parts and those later parts read from
/// them.
fn part(
  model: &ModelProto,
  graph: &GraphProto,
  types: &HashMap<&str, i32>,
  from: usize,
  to: usize,
) -> (Vec<u8>, Vec<String>, Vec<String>) {
  let nodes = &graph.node[from..to];
  let initializers: HashSet<&str> = graph.initializer.iter().map(|t| t.name.as_str()).collect();
  let made: HashSet<&str> = nodes
    .iter()
    .flat_map(|node| node.output.iter().map(String::as_str))
    .collect();
  let mut inputs: Vec<String> = Vec::new();
  let mut kept: HashSet<&str> = HashSet::new();
  for name in nodes.iter().flat_map(|node| &node.input) {
    if name.is_empty() || made.contains(name.as_str()) {
      continue;
    }
    if initializers.contains(name.as_str()) {
      kept.insert(name);
    } else if !inputs.contains(name) {
      inputs.push(name.clone());
    }
  }
  let read_later: HashSet<&str> = graph.node[to..]
    .iter()
    .flat_map(|node| node.input.iter().map(String::as_str))
    .chain(graph.output.iter().map(|output| output.name.as_str()))
    .collect();
  let outputs: Vec<String> = nodes
    .iter()
    .flat_map(|node| &node.output)
    .filter(|name| read_later.contains(name.as_str()))
    .cloned()
    .collect();
  let typed = |name: &String| ValueInfoProto {
    name: name.clone(),
    r#type: Some(TypeProto {
      denotation: String::new(),
      value: Some(type_proto::Value::TensorType(type_proto::Tensor {
        elem_type: types.get(name.as_str()).copied().unwrap_or(FLOAT),
        shape: None,
      })),
    }),
    doc_string: String::new(),
  };
  // The header alone is copied: cloning the whole model would copy every
  // weight.
  let part = ModelProto {
    ir_version: model.ir_version,
    opset_import: model.opset_import.clone(),
    producer_name: model.producer_name.clone(),
    producer_version: model.producer_version.clone(),
    domain: model.domain.clone(),
    model_version: model.model_version,
    doc_string: String::new(),
    graph: Some(GraphProto {
      name: graph.name.clone(),
      node: nodes.to_vec(),
      initializer: graph
        .initializer
        .iter()
        .filter(|tensor| kept.contains(tensor.name.as_str()))
        .cloned()
        .collect(),
      input: inputs.iter().map(typed).collect(),
      output: outputs.iter().map(typed).collect(),
      ..GraphProto::default()
    }),
    metadata_props: Vec::new(),
    training_info: Vec::new(),
    functions: model.functions.clone(),
  };
  (part.encode_to_vec(), inputs, outputs)
}

const FLOAT: i32 = 1;
const INT64: i32 = 7;
const BOOL: i32 = 9;

/// Each tensor's element type, as each operator makes it. A part's inputs
/// and outputs must declare theirs, and the model records none.
fn element_types(graph: &GraphProto) -> HashMap<&str, i32> {
  let mut types: HashMap<&str, i32> = graph
    .input
    .iter()
    .map(|input| (input.name.as_str(), FLOAT))
    .chain(
      graph
        .initializer
        .iter()
        .map(|tensor| (tensor.name.as_str(), tensor.data_type)),
    )
    .collect();
  for node in &graph.node {
    let attribute = |name: &str| node.attribute.iter().find(|a| a.name == name);
    let type_of = |name: &str| types.get(name).copied();
    let kind = match node.op_type.as_str() {
      "Shape" | "Size" | "NonZero" | "ArgMax" | "ArgMin" => INT64,
      "Cast" => attribute("to").map_or(FLOAT, |to| i32::try_from(to.i).unwrap_or(FLOAT)),
      "Constant" | "ConstantOfShape" => attribute("value")
        .and_then(|value| value.t.as_ref())
        .map_or(FLOAT, |tensor| tensor.data_type),
      "Equal" | "Less" | "Greater" | "LessOrEqual" | "GreaterOrEqual" | "Not" | "And" | "Or"
      | "IsNaN" | "IsInf" => BOOL,
      "Where" => node
        .input
        .get(1)
        .and_then(|name| type_of(name))
        .unwrap_or(FLOAT),
      _ => node
        .input
        .iter()
        .find(|name| !name.is_empty())
        .and_then(|name| type_of(name))
        .unwrap_or(FLOAT),
    };
    for output in &node.output {
      types.insert(output, kind);
    }
  }
  types
}
