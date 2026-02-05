use crate::render_graph::{dag::DirectedAcyclicGraph, gpu_operation::Operation};

/// This struct purpose is to optimize rendering with batching, optimal synchronization usage, parallelization and culling 
pub struct RenderGraph {
    dag: DirectedAcyclicGraph<Operation, IntermediateAction>,
    target_node: usize,
}
impl RenderGraph {
    pub fn new() -> Self {
        Self {
            dag: DirectedAcyclicGraph::new(),
            target_node: 0,
        }
    }
    /// Inserts operation into render graph
    /// 
    /// If operation is not infuencing any target operations it would be culled at compilation 
    pub fn add_operation(&mut self, op: Operation) {
        self.dag.add_node(op);
    }

    /// Inserts operation into render graph marked as targeted, `target` operations are the ones that define output of app 
    pub fn add_target_op(&mut self, target: Operation) {
        self.dag.add_node(target);
        self.target_node = self.dag.node_count() - 1;
    }

    /// This function is main feature of render graph
    pub fn compile(&mut self) -> Option<Vec<Operation>> {
        let nodes = self.dag.nodes().clone();

        let mut stack = vec![self.target_node];

        //Fills in graph: creates edges in graph based on whetever operation or any of it children makes influence onto the target's operation read_resources()   
        while let Some(current) = stack.pop() {
            let current_node = nodes[current];
            for (node_id, node) in nodes.iter().enumerate() {
                if node
                    .write_resources()
                    .iter()
                    .any(|x| current_node.read_resources().contains(x))
                {
                    stack.push(node_id);
                    self.dag
                        .add_edge_cyclic(node_id, current, IntermediateAction::None);
                }
            }
        }
        //Removes nodes that are not influencing on target nodes and makes topological sort (linearizing operation order) 
        if let Some(compiled) = self.dag.compile(self.target_node) {
            return Some(
                compiled
                    .iter()
                    .cloned()
                    .map(|x| self.dag.get_node(x).cloned().unwrap())
                    .collect(),
            );
        }
        None
    }
    pub fn dag(&self) -> &DirectedAcyclicGraph<Operation, IntermediateAction> {
        &self.dag
    }
    ///This operation is intented to use after all work is done
    pub fn clear(&mut self) {
        self.dag.clear();
    }
}
#[derive(Debug, Clone, Copy)]
pub enum IntermediateAction {
    None,
}
#[cfg(test)]
pub mod test {
    use crate::{
        render_graph::{
            self, gpu_operation::Operation, render_graph::RenderGraph, resource::ResourceId,
        },
        rendering::texture_container::{self, CreateTexture, CreateTextureView},
    };

    #[test]
    pub fn test_basic() {
        let mut rendergraph = RenderGraph::new();
        let mut texture_container = texture_container::TextureContainer::new();
        let present = texture_container.create_texture_view_null();
        let source = texture_container.create_texture_view_null();
        let intermid = texture_container.create_texture_view_null();

        let present_op = Operation::Present(ResourceId::Texture(present));
        let draw_op =
            Operation::DrawCall(ResourceId::Texture(intermid), ResourceId::Texture(present));
        let draw_op1 =
            Operation::DrawCall(ResourceId::Texture(source), ResourceId::Texture(intermid));
        rendergraph.add_target_op(present_op);
        rendergraph.add_operation(draw_op);
        rendergraph.add_operation(draw_op1);
        assert_eq!(
            Some(vec![draw_op1, draw_op, present_op]),
            rendergraph.compile()
        );
        println!("{:?}", rendergraph.compile());
    }
    #[test]
    pub fn test_parallel() {
        let mut rendergraph = RenderGraph::new();
        let mut texture_container = texture_container::TextureContainer::new();
        let present = texture_container.create_texture_view_null();
        let source = texture_container.create_texture_view_null();
        let source_2_inter = texture_container.create_texture_view_null();
        let source_2 = texture_container.create_texture_view_null();
        println!(
            "present:{:?} source:{:?} source_2_inter:{:?} source_2:{:?}",
            present, source, source_2_inter, source_2
        );
        let present_op = Operation::Present(ResourceId::Texture(present));
        let draw_op =
            Operation::DrawCall(ResourceId::Texture(source), ResourceId::Texture(present));
        let draw_op1 = Operation::DrawCall(
            ResourceId::Texture(source_2_inter),
            ResourceId::Texture(present),
        );
        let dep_op1 = Operation::DrawCall(
            ResourceId::Texture(source_2),
            ResourceId::Texture(source_2_inter),
        );
        rendergraph.add_target_op(present_op);
        rendergraph.add_operation(draw_op);
        rendergraph.add_operation(draw_op1);
        rendergraph.add_operation(dep_op1);
        assert_eq!(
            Some(vec![draw_op, dep_op1, draw_op1, present_op]),
            rendergraph.compile()
        );
    }
}
