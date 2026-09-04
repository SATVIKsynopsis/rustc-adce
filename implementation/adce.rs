use rustc_middle::mir::*;
use rustc_middle::ty::TyCtxt;
use rustc_session::Session;
use rustc_mir_dataflow::impls::{MaybeTransitiveLiveLocals};
use rustc_mir_dataflow::debuginfo::debuginfo_locals;
//use rustc_mir_dataflow::{Analysis, ResultsVisitor, visit_results};
//use rustc_index::bit_set::DenseBitSet;
use rustc_data_structures::graph::dominators::Dominators;
use crate::simplify::SimplifyCfg;
use rustc_mir_dataflow::impls::borrowed_locals;
use rustc_mir_dataflow::Analysis;

use crate::{MirPass, PassPolicy};

pub(crate) struct AdcePass;

fn find_control_dependent_blocks(
    branch_block: BasicBlock,
    successor: BasicBlock,
    post_doms: &Dominators<BasicBlock>,
) -> Option<Vec<BasicBlock>> {
    let mut result = Vec::new();
    let mut current = successor;

    loop {
        if !post_doms.is_reachable(current) {
            return None;
        }
        if post_doms.dominates(current, branch_block) {
            break;
        }
        result.push(current);
        match post_doms.immediate_dominator(current) {
            Some(next) => current = next,
            None => break,
        }
    }
    Some(result)
}

fn branch_is_dead(
    body: &Body<'_>,
    branch_block: BasicBlock,
    post_doms: &Dominators<BasicBlock>,
) -> bool {
    let successors: Vec<_> = body.basic_blocks[branch_block]
        .terminator()
        .successors()
        .collect();

    if successors.len() <= 1 {
        return false;
    }

    if !post_doms.is_reachable(branch_block) {
        return false;
    }

    for successor in successors {
        let Some(dependent) = find_control_dependent_blocks(branch_block, successor, post_doms) else {
            return false;
        };

        for bb in dependent {
            let data = &body.basic_blocks[bb];
            let effectively_empty = data.statements.iter().all(|s| matches!(
                s.kind,
                StatementKind::Nop
                    | StatementKind::StorageLive(_)
                    | StatementKind::StorageDead(_)
            ));

            if !effectively_empty {
                return false;
            }
            if !matches!(data.terminator().kind, TerminatorKind::Goto { .. }) {
                return false;
            }
        }
    }

    true
}

impl<'tcx> MirPass<'tcx> for AdcePass {
    fn policy(&self, _sess: &Session) -> PassPolicy {
        PassPolicy::optimization(true)
    }

    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        //Phase 1: control-dependence-based dead branch collapse.
        let post_dom = crate::post_dom::PostDomGraph::new(body);
        let doms = rustc_data_structures::graph::dominators::dominators(&post_dom);

        for bb in body.basic_blocks.indices() {
            let successors: Vec<_> = body.basic_blocks[bb].terminator().successors().collect();
            if successors.len() > 1 && branch_is_dead(body, bb, &doms) {
                let merge = doms
                    .immediate_dominator(bb)
                    .expect("dead branch must have a post-dominator");
                if body.basic_blocks[bb].is_cleanup != body.basic_blocks[merge].is_cleanup {
                    continue;
                }
                body.basic_blocks_mut()[bb].terminator_mut().kind =
                    TerminatorKind::Goto { target: merge };
            }
        }

        SimplifyCfg::Final.run_pass(tcx, body);

        // Phase 2: value-liveness-based dead statement removal.
        let borrowed_locals = borrowed_locals(body);
        let debuginfo_locals = debuginfo_locals(body);

        let analysis =
            MaybeTransitiveLiveLocals::new(&borrowed_locals, &debuginfo_locals);

        let mut live = analysis
            .iterate_to_fixpoint(tcx, body, None)
            .into_results_cursor(body);

        let mut dead_locations = Vec::new();

        for (bb, bb_data) in traversal::preorder(body) {
            for (statement_index, statement) in bb_data.statements.iter().enumerate().rev() {
                let StatementKind::Assign(assign) = &statement.kind else {
                    continue;
                };

                let (place, rvalue) = &**assign;
                let Some(local) = place.as_local() else {
                    continue;
                };

                // Keep ADCE Phase 2 intentionally narrower than DSE.
                if borrowed_locals.contains(local) || debuginfo_locals.contains(local) {
                    continue;
                }

                if !matches!(rvalue, Rvalue::Use(..)) {
                    continue;
                }

                let loc = Location {
                    block: bb,
                    statement_index,
                };

                // This is the important part copied from real DSE.
                live.seek_before_primary_effect(loc);

                if !live.get().contains(local) {
                    dead_locations.push(loc);
                }
            }
        }

        // Cursor no longer borrows body.
        for loc in dead_locations {
            body.basic_blocks_mut()[loc.block]
                .statements[loc.statement_index]
                .make_nop(true);
        }
    }
}

// struct DeadStatementCollector<'a> {
//     always_live: &'a DenseBitSet<Local>,
//     debuginfo: &'a DenseBitSet<Local>,
//     dead_locations: Vec<Location>,
// }

// impl<'a, 'tcx> ResultsVisitor<'tcx, MaybeTransitiveLiveLocals<'_>> for DeadStatementCollector<'a> {
//     fn visit_after_primary_statement_effect(
//         &mut self,
//         state: &DenseBitSet<Local>,
//         statement: &Statement<'tcx>,
//         location: Location,
//     ) {
//         let StatementKind::Assign(assign) = &statement.kind else { return };
//         let (place, rvalue) = &**assign;
//         let Some(local) = place.as_local() else { return };

//         if self.always_live.contains(local) || self.debuginfo.contains(local) {
//             return;
//         }
//         if !matches!(rvalue, Rvalue::Use(..)) {
//             return;
//         }
//         if !state.contains(local) {
//             self.dead_locations.push(location);
//         }
//     }
// }