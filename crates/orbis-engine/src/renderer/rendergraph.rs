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
enum EdgeKind {
    ReadAfterWrite,
    WriterOrder,
}

#[derive(Clone, Copy)]
struct Edge {
    from: usize,
    to: usize,
    resource: ResourceId,
    kind: EdgeKind,
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

            // Readers depend on every writer so all writes complete first.
            for &writer in &res.writers {
                for &reader in &res.readers {
                    if reader != writer {
                        deps[reader].push(Edge {
                            from: reader,
                            to: writer,
                            resource: res.id,
                            kind: EdgeKind::ReadAfterWrite,
                        });
                    }
                }
            }

            // Multiple writers are intentionally ordered by pass registration.
            // This preserves the renderer's existing accumulation/compositing order.
            for writers in res.writers.windows(2) {
                let earlier = writers[0];
                let later = writers[1];
                if earlier != later {
                    deps[later].push(Edge {
                        from: later,
                        to: earlier,
                        resource: res.id,
                        kind: EdgeKind::WriterOrder,
                    });
                }
            }
        }

        for edges in &mut deps {
            edges.sort_by(|left, right| {
                left.resource
                    .to_string()
                    .cmp(&right.resource.to_string())
                    .then_with(|| left.to.cmp(&right.to))
            });
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
                    let cycle_start = stack
                        .iter()
                        .position(|edge| edge.from == i)
                        .unwrap_or(0);
                    let cycle_lines = stack[cycle_start..]
                        .iter()
                        .map(|edge| {
                            match edge.kind {
                                EdgeKind::ReadAfterWrite => format!(
                                    "{} reads {} -> depends on {}",
                                    passes[edge.from].name(),
                                    edge.resource,
                                    passes[edge.to].name()
                                ),
                                EdgeKind::WriterOrder => format!(
                                    "{} writes {} after {}",
                                    passes[edge.from].name(),
                                    edge.resource,
                                    passes[edge.to].name()
                                ),
                            }
                        })
                        .collect::<Vec<_>>();

                    return Err(format!("Cycle detected:\n{}", cycle_lines.join("\n")));
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
        let skyboxpass = SkyboxPass {};
        let build_mipmaps = BuildMipmapsPass {};
        let transmission = MeshPass::transmission();
        let lightpass = LightsIconPass {};
        let axispass = AxisPass {};
        let bboxpass = LinesPass {};
        let linearizepass = LinearizePass {};
        let outlinepass = OutlinePass {};

        graph.add_pass(meshpass);
        graph.add_pass(skyboxpass);
        graph.add_pass(build_mipmaps);
        graph.add_pass(transmission);
        graph.add_pass(lightpass);
        graph.add_pass(axispass);
        graph.add_pass(bboxpass);
        graph.add_pass(linearizepass);
        graph.add_pass(outlinepass);

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
                    ]
                );
            }
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn dependencies_compile_writer_before_reader() {
        struct Writer;
        struct Reader;

        impl RenderPass for Writer {
            fn name(&self) -> &'static str {
                "Writer"
            }

            fn reads(&self) -> &[ResourceId] {
                &[]
            }

            fn writes(&self) -> &[ResourceId] {
                &[ResourceId::HDR]
            }
        }

        impl RenderPass for Reader {
            fn name(&self) -> &'static str {
                "Reader"
            }

            fn reads(&self) -> &[ResourceId] {
                &[ResourceId::HDR]
            }

            fn writes(&self) -> &[ResourceId] {
                &[]
            }
        }

        let mut graph = RenderGraph::new();
        graph.add_pass(Reader);
        graph.add_pass(Writer);

        let order = graph.compile_names().unwrap();
        let writer = order.iter().position(|name| *name == "Writer").unwrap();
        let reader = order.iter().position(|name| *name == "Reader").unwrap();

        assert!(writer < reader, "writer must run before its reader: {order:?}");
    }

    #[test]
    fn dependency_order_is_deterministic() {
        struct Writer(ResourceId, &'static str);
        struct Reader(ResourceId, &'static str);

        impl RenderPass for Writer {
            fn name(&self) -> &'static str {
                self.1
            }

            fn reads(&self) -> &[ResourceId] {
                &[]
            }

            fn writes(&self) -> &[ResourceId] {
                std::slice::from_ref(&self.0)
            }
        }

        impl RenderPass for Reader {
            fn name(&self) -> &'static str {
                self.1
            }

            fn reads(&self) -> &[ResourceId] {
                std::slice::from_ref(&self.0)
            }

            fn writes(&self) -> &[ResourceId] {
                &[]
            }
        }

        let mut graph = RenderGraph::new();
        graph.add_pass(Reader(ResourceId::LDR, "Reader LDR"));
        graph.add_pass(Reader(ResourceId::HDR, "Reader HDR"));
        graph.add_pass(Writer(ResourceId::LDR, "Writer LDR"));
        graph.add_pass(Writer(ResourceId::HDR, "Writer HDR"));

        let expected = graph.compile_names().unwrap();
        for _ in 0..32 {
            assert_eq!(graph.compile_names().unwrap(), expected);
        }
    }


    #[test]
    fn multiple_writers_preserve_registration_order() {
        struct Pass {
            name: &'static str,
            reads: Vec<ResourceId>,
            writes: Vec<ResourceId>,
        }

        impl RenderPass for Pass {
            fn name(&self) -> &'static str {
                self.name
            }

            fn reads(&self) -> &[ResourceId] {
                &self.reads
            }

            fn writes(&self) -> &[ResourceId] {
                &self.writes
            }
        }

        let mut graph = RenderGraph::new();
        graph.add_pass(Pass {
            name: "First writer",
            reads: vec![],
            writes: vec![ResourceId::HDR],
        });
        graph.add_pass(Pass {
            name: "Second writer",
            reads: vec![],
            writes: vec![ResourceId::HDR],
        });
        graph.add_pass(Pass {
            name: "Reader",
            reads: vec![ResourceId::HDR],
            writes: vec![],
        });

        assert_eq!(
            graph.compile_names().unwrap(),
            vec!["First writer", "Second writer", "Reader"]
        );
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

        let error = graph.compile().unwrap_err();

        assert!(error.contains("Pass A reads LDR -> depends on Pass B"));
        assert!(error.contains("Pass B reads Depth -> depends on Pass A"));
        assert_eq!(error.lines().count(), 3, "diagnostic should contain only the cycle");
    }
}
