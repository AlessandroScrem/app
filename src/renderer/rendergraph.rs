#![allow(dead_code)]

use std::collections::HashMap;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct ResourceId(pub u32);

type RenderContext = u32;

pub const ENTITY: ResourceId = ResourceId(1);
pub const DEPTH: ResourceId = ResourceId(2);
pub const HDRA: ResourceId = ResourceId(3);
pub const HDRB: ResourceId = ResourceId(4);
pub const LDR: ResourceId = ResourceId(5);
pub const PICKBUFFER: ResourceId = ResourceId(6);
pub const LIGHTTEXTURE: ResourceId = ResourceId(6);

#[derive(Clone, Copy)]
enum VisitState {
    NotVisited,
    Visiting, // ← in stack node
    Visited,
}

pub trait RenderPassNode {
    fn name(&self) -> &str;

    fn reads(&self) -> &[ResourceId];
    fn writes(&self) -> &[ResourceId];

    fn execute(&self, ctx: &mut RenderContext);
}

// =========================
// Graph internals
// =========================

#[derive(Debug)]
struct ResourceNode {
    id: ResourceId,
    writer: Option<usize>,
    readers: Vec<usize>,
}

struct PassNode<'a> {
    pass: &'a dyn RenderPassNode,
    reads: Vec<ResourceId>,
    writes: Vec<ResourceId>,
}

#[derive(Clone, Copy)]
struct Edge {
    to: usize,
    resource: ResourceId,
}

// =========================
// RenderGraph
// =========================

struct RenderGraph {
    passes: Vec<Box<dyn RenderPassNode>>,
}

impl RenderGraph {
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }

    pub fn add_pass<P: RenderPassNode + 'static>(&mut self, pass: P) {
        self.passes.push(Box::new(pass));
    }

    // -------------------------
    // Build Pass + Resource graph
    // -------------------------
    fn build_graph(
        &self,
    ) -> (
        Vec<PassNode<'_>>,
        HashMap<ResourceId, ResourceNode>,
        Vec<String>,
    ) {
        let mut resources: HashMap<ResourceId, ResourceNode> = HashMap::new();
        let mut errors = Vec::new();

        let pass_nodes: Vec<_> = self
            .passes
            .iter()
            .map(|p| PassNode {
                pass: p.as_ref(),
                reads: p.reads().to_vec(),
                writes: p.writes().to_vec(),
            })
            .collect();

        for (i, pass) in pass_nodes.iter().enumerate() {
            // writes
            for &res in &pass.writes {
                let entry = resources.entry(res).or_insert(ResourceNode {
                    id: res,
                    writer: None,
                    readers: Vec::new(),
                });

                if let Some(prev) = entry.writer {
                    errors.push(format!(
                        "❌ Multiple writers for resource {:?}: {} -> {}",
                        res,
                        pass_nodes[prev].pass.name(),
                        pass.pass.name()
                    ));
                }

                entry.writer = Some(i);
            }

            // reads
            for &res in &pass.reads {
                let entry = resources.entry(res).or_insert(ResourceNode {
                    id: res,
                    writer: None,
                    readers: Vec::new(),
                });

                entry.readers.push(i);
            }
        }

        (pass_nodes, resources, errors)
    }

    // -------------------------
    // Build dependencies (with resource info)
    // -------------------------
    fn build_dependencies_internal(
        &self,
        passes: &Vec<PassNode>,
        resources: &HashMap<ResourceId, ResourceNode>,
    ) -> Vec<Vec<Edge>> {
        let mut deps = vec![Vec::new(); passes.len()];

        for res in resources.values() {
            if res.writer.is_none() && !res.readers.is_empty() {
                println!("⚠️ Resource {:?} read but never written", res.id);
            }

            if let Some(writer) = res.writer {
                for &reader in &res.readers {
                    deps[reader].push(Edge {
                        to: writer,
                        resource: res.id,
                    });
                }
            }
        }

        deps
    }

    // -------------------------
    // Compile (topo sort + cycle detection)
    // Topological sort (DFS)
    // -------------------------
    pub fn compile(&self) -> Result<Vec<usize>, String> {
        let (passes, resources, errors) = self.build_graph();

        if !errors.is_empty() {
            return Err(errors.join("\n"));
        }

        let deps = self.build_dependencies_internal(&passes, &resources);

        let mut state = vec![VisitState::NotVisited; self.passes.len()];
        let mut result = Vec::new();
        let mut stack: Vec<(usize, Option<ResourceId>)> = Vec::new();

        fn visit(
            i: usize,
            deps: &Vec<Vec<Edge>>,
            state: &mut Vec<VisitState>,
            stack: &mut Vec<(usize, Option<ResourceId>)>,
            result: &mut Vec<usize>,
            passes: &Vec<Box<dyn RenderPassNode>>,
        ) -> Result<(), String> {
            match state[i] {
                VisitState::Visited => return Ok(()),

                VisitState::Visiting => {
                    // build readable cycle
                    let mut cycle = Vec::new();

                    for (idx, res) in stack.iter() {
                        if let Some(r) = res {
                            cycle.push(format!("{} --({:?})->", passes[*idx].name(), r));
                        } else {
                            cycle.push(passes[*idx].name().to_string());
                        }
                    }

                    cycle.push(passes[i].name().to_string());

                    return Err(format!("❌ Cycle detected:\n{}", cycle.join(" ")));
                }

                VisitState::NotVisited => {}
            }

            state[i] = VisitState::Visiting;

            for edge in &deps[i] {
                stack.push((i, Some(edge.resource)));

                visit(edge.to, deps, state, stack, result, passes)?;

                stack.pop();
            }

            state[i] = VisitState::Visited;
            result.push(i);

            Ok(())
        }

        for i in 0..self.passes.len() {
            visit(i, &deps, &mut state, &mut stack, &mut result, &self.passes)?;
        }

        Ok(result)
    }

    // -------------------------
    // Execute with no failure
    // -------------------------
    pub fn execute(&self, ctx: &mut RenderContext) {
        match self.compile() {
            Err(e) => println!("{}", e),
            Ok(order) => {
                for idx in order {
                    let pass = &self.passes[idx];
                    pass.execute(ctx);
                }
            }
        }
    }
}

impl RenderGraph {
    pub fn compile_names(&self) -> Result<Vec<String>, String> {
        let order = self.compile()?;

        Ok(order
            .into_iter()
            .map(|i| self.passes[i].name().to_string())
            .collect())
    }
}


struct GeometryPass;

impl RenderPassNode for GeometryPass {
    fn name(&self) -> &str {
        "Geometry"
    }

    fn reads(&self) -> &[ResourceId] {
        &[]
    }

    fn writes(&self) -> &[ResourceId] {
        &[ENTITY, DEPTH]
    }

    fn execute(&self, _ctx: &mut RenderContext) {
        println!("Geometry pass");
    }
}
struct LightingPass;

impl RenderPassNode for LightingPass {
    fn name(&self) -> &str {
        "Lighting"
    }

    fn reads(&self) -> &[ResourceId] {
        &[ENTITY, DEPTH]
    }

    fn writes(&self) -> &[ResourceId] {
        &[HDRA]
    }

    fn execute(&self, _ctx: &mut RenderContext) {
        println!("Lighting pass");
    }
}
struct TransmissionPass;

impl RenderPassNode for TransmissionPass {
    fn name(&self) -> &str {
        "Transmission"
    }

    fn reads(&self) -> &[ResourceId] {
        &[HDRA, DEPTH]
    }

    fn writes(&self) -> &[ResourceId] {
        &[HDRB]
    }

    fn execute(&self, _ctx: &mut RenderContext) {
        println!("Transmission pass");
    }
}
struct TonemapPass;

impl RenderPassNode for TonemapPass {
    fn name(&self) -> &str {
        "Tonemap"
    }

    fn reads(&self) -> &[ResourceId] {
        &[HDRB]
    }

    fn writes(&self) -> &[ResourceId] {
        &[LDR]
    }

    fn execute(&self, _ctx: &mut RenderContext) {
        println!("Tonemap pass");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_run() {
        let mut graph = RenderGraph::new();
        let transmission_enabled = true;

        graph.add_pass(GeometryPass);
        graph.add_pass(LightingPass);

        if transmission_enabled {
            graph.add_pass(TransmissionPass);
        }

        graph.add_pass(TonemapPass);

        assert!(graph.compile().is_ok());
    }

    #[test]
    fn should_find_cycles() {
        struct PassA;
        struct PassB;

        impl RenderPassNode for PassA {
            fn name(&self) -> &str {
                "Pass A"
            }
            fn reads(&self) -> &[ResourceId] {
                &[LDR]
            }
            fn writes(&self) -> &[ResourceId] {
                &[DEPTH]
            }
            fn execute(&self, _ctx: &mut RenderContext) {
                println!("Pass A pass");
            }
        }

        impl RenderPassNode for PassB {
            fn name(&self) -> &str {
                "Pass B"
            }
            fn reads(&self) -> &[ResourceId] {
                &[DEPTH]
            }
            fn writes(&self) -> &[ResourceId] {
                &[LDR]
            }
            fn execute(&self, _ctx: &mut RenderContext) {
                println!("Pass B pass");
            }
        }

        let mut graph = RenderGraph::new();
        let mut ctx: RenderContext = 0;

        graph.add_pass(PassA);
        graph.add_pass(PassB);

        assert!(graph.compile().is_err());

        graph.execute(&mut ctx);
    }
}
