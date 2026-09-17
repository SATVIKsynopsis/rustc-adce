use rustc_data_structures::graph::{
    DirectedGraph, Predecessors, StartNode, Successors,
};
use rustc_middle::mir::{BasicBlock, Body};

pub(crate) struct PostDomGraph<'a, 'tcx> {
    body: &'a Body<'tcx>,
    virtual_exit: BasicBlock,
    real_exits: Vec<BasicBlock>,
}

impl<'a, 'tcx> PostDomGraph<'a, 'tcx> {
    pub(crate) fn new(body: &'a Body<'tcx>) -> Self {
        let real_exits = body
    .basic_blocks
    .iter_enumerated()
    .filter_map(|(bb, data)| {
        // A "real exit" is any block with no successors at all — this
        // naturally covers Return, Unreachable, UnwindResume, AND any
        // diverging call/drop/assert whose terminator has no target
        // and an unwind action that doesn't point at another block
        // (UnwindAction::Continue or ::Terminate).
        if data.terminator().successors().next().is_none() {
            Some(bb)
        } else {
            None
        }
    })
    .collect();

        let virtual_exit = BasicBlock::from_usize(body.basic_blocks.len());

        Self {
            body,
            virtual_exit,
            real_exits,
        }
    }
}

impl DirectedGraph for PostDomGraph<'_, '_> {
    type Node = BasicBlock;

    fn num_nodes(&self) -> usize {
        self.body.basic_blocks.len() + 1
    }
}

impl StartNode for PostDomGraph<'_, '_> {
    fn start_node(&self) -> BasicBlock {
        self.virtual_exit
    }
}

impl Successors for PostDomGraph<'_, '_> {
    fn successors(&self, node: BasicBlock) -> impl Iterator<Item = BasicBlock> {
        if node == self.virtual_exit {
            return Box::new(self.real_exits.iter().copied())
                as Box<dyn Iterator<Item = BasicBlock>>;
        }

        Box::new(
            self.body.basic_blocks
                .predecessors()[node]
                .iter()
                .copied(),
        )
    }
}

impl Predecessors for PostDomGraph<'_, '_> {
    fn predecessors(&self, node: BasicBlock) -> impl Iterator<Item = BasicBlock> {
        if self.real_exits.contains(&node) {
            return Box::new(
                std::iter::once(self.virtual_exit)
                    .chain(self.body.basic_blocks[node].terminator().successors()),
            ) as Box<dyn Iterator<Item = BasicBlock>>;
        }

        Box::new(
            self.body.basic_blocks[node]
                .terminator()
                .successors(),
        )
    }
}

#[cfg(test)]
mod tests {
    // nothing yet
}