#![allow(dead_code)]

use super::*;
use std::collections::HashMap;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum ResourceId {
    ENTITY,
    DEPTH,
    HDR,
    LDR,
    OPAQUE,
    PICKBUFFER,
    SHADOWMAP,
}

use std::fmt;
impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match *self {
            Self::HDR => "HDR",
            Self::OPAQUE => "Opaque",
            Self::DEPTH => "Depth",
            Self::ENTITY => "EntityID",
            Self::LDR => "LDR",
            Self::PICKBUFFER => "PickBuffer",
            Self::SHADOWMAP => "ShadowMap",
        };
        write!(f, "{}", name)
    }
}

impl fmt::Debug for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Clone, Copy)]
enum VisitState {
    NotVisited,
    Visiting, // ← in stack node
    Visited,
}

// =========================
// Graph internals
// =========================

#[derive(Debug)]
struct ResourceNode {
    id: ResourceId,
    writers: Vec<usize>,
    readers: Vec<usize>,
}

struct PassNode<'a> {
    pass: &'a dyn RenderPass,
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
    passes: Vec<Box<dyn RenderPass>>,
}

impl RenderGraph {
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }

    pub fn add_pass<P: RenderPass + 'static>(&mut self, pass: P) {
        self.passes.push(Box::new(pass));
    }

    // -------------------------
    // Build Pass + Resource graph
    // -------------------------
    fn build_graph(&self) -> (Vec<PassNode<'_>>, HashMap<ResourceId, ResourceNode>) {
        let mut resources: HashMap<ResourceId, ResourceNode> = HashMap::new();

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
                    writers: Vec::new(),
                    readers: Vec::new(),
                });

                entry.writers.push(i);
            }

            // reads
            for &res in &pass.reads {
                let entry = resources.entry(res).or_insert(ResourceNode {
                    id: res,
                    writers: Vec::new(),
                    readers: Vec::new(),
                });

                entry.readers.push(i);
            }
        }

        (pass_nodes, resources)
    }

    // -------------------------
    // Build dependencies (with resource info)
    // -------------------------
    fn build_dependencies(
        &self,
        passes: &Vec<PassNode>,
        resources: &HashMap<ResourceId, ResourceNode>,
    ) -> Vec<Vec<Edge>> {
        let mut deps = vec![Vec::new(); passes.len()];
        for res in resources.values() {
            if res.writers.is_empty() && !res.readers.is_empty() {
                println!("⚠️ Resource {} read but never written", res.id);
            }

            //For each writer and for each reader make one edge
            for &writer in &res.writers {
                for &reader in &res.readers {
                    if reader != writer {
                        deps[reader].push(Edge {
                            to: writer,
                            resource: res.id,
                        });
                    }
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
        let (passes, resources) = self.build_graph();

        let deps = self.build_dependencies(&passes, &resources);

        let mut state = vec![VisitState::NotVisited; self.passes.len()];
        let mut result = Vec::new();
        let mut stack: Vec<Edge> = Vec::new();

        fn visit(
            i: usize,
            deps: &Vec<Vec<Edge>>,
            state: &mut Vec<VisitState>,
            stack: &mut Vec<Edge>,
            result: &mut Vec<usize>,
            passes: &Vec<Box<dyn RenderPass>>,
        ) -> Result<(), String> {
            match state[i] {
                VisitState::Visited => return Ok(()),
                VisitState::Visiting => {
                    // Costruisci il ciclo leggibile
                    let mut cycle_lines = Vec::new();

                    for edge in stack.iter() {
                        cycle_lines.push(format!(
                            "{} reads {} -> depends on {}",
                            passes[edge.to].name(), // chi scrive la risorsa
                            edge.resource,
                            passes[edge.to].name() // writer corretto
                        ));
                    }

                    return Err(format!("❌ Cycle detected:\n{}", cycle_lines.join("\n")));
                }
                VisitState::NotVisited => {}
            }

            state[i] = VisitState::Visiting;

            for edge in &deps[i] {
                stack.push(*edge);
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
    pub fn execute(&self) {
        match self.compile() {
            Err(e) => println!("{}", e),
            Ok(order) => {
                for idx in order {
                    let pass = &self.passes[idx];
                    println!("{}", pass.name());
                    // pass.execute(ctx);
                }
            }
        }
    }
}

impl RenderGraph {
    fn compile_names(&self) -> Result<Vec<String>, String> {
        let order = self.compile()?;

        Ok(order
            .into_iter()
            .map(|i| self.passes[i].name().to_string())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_run_in_order() {
        let mut graph = RenderGraph::new();
        let meshpass = MeshPass::opaque();
        let skyboxpass = SkyboxPass::new();
        let build_mipmaps = BuildMipmapsPass::new();
        let transmission = MeshPass::transmission();
        let lightpass = LightsIconPass::new();
        let axispass = AxisPass::new();
        let bboxpass = LinesPass::new();
        let linearizepass = LinearizePass::new();
        let outlinepass = OutlinePass::new();
        let pickobjectpass = PickObjectPass::new();

        graph.add_pass(meshpass);
        graph.add_pass(skyboxpass);
        graph.add_pass(build_mipmaps);
        graph.add_pass(transmission);
        graph.add_pass(lightpass);
        graph.add_pass(axispass);
        graph.add_pass(bboxpass);
        graph.add_pass(linearizepass);
        graph.add_pass(outlinepass);
        graph.add_pass(pickobjectpass);

        match graph.compile_names() {
            Ok(order) => {
                println!("Order: {:?}", order);
                assert_eq!(
                    order,
                    vec![
                        "MeshPass Opaque",
                        "SkyboxPass",
                        "BuildMipmapsPass",
                        "MeshPass Transmission",
                        "LightPass",
                        "AxisPass",
                        "BoundingboxPass",
                        "LinearizePass",
                        "OutlinePass",
                        "PickObjectPass"
                    ]
                );
            }
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn should_find_cycles() {
        struct PassA;
        struct PassB;

        impl RenderPass for PassA {
            fn name(&self) -> &'static str {
                "Pass A"
            }

            fn reads(&self) -> &[ResourceId] {
                &[ResourceId::LDR]
            }
            fn writes(&self) -> &[ResourceId] {
                &[ResourceId::DEPTH]
            }
        }

        impl RenderPass for PassB {
            fn name(&self) -> &'static str {
                "Pass B"
            }
            fn reads(&self) -> &[ResourceId] {
                &[ResourceId::DEPTH]
            }
            fn writes(&self) -> &[ResourceId] {
                &[ResourceId::LDR]
            }
        }

        let mut graph = RenderGraph::new();

        graph.add_pass(PassA);
        graph.add_pass(PassB);

        assert!(graph.compile().is_err());
        graph.execute();
    }
}
