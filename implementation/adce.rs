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
use rustc_data_structures::fx::FxHashSet;
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
    dead_set: &FxHashSet<Location>,
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
        let Some(dependent) = find_control_dependent_blocks(branch_block, successor, post_doms)
        else {
            return false;
        };
        for bb in dependent {
            let data = &body.basic_blocks[bb];
            if !matches!(data.terminator().kind, TerminatorKind::Goto { .. }) {
                return false;
            }
            for statement_index in 0..data.statements.len() {
                let stmt = &data.statements[statement_index];

                let ok = match &stmt.kind {
                    StatementKind::Nop
                    | StatementKind::StorageLive(_)
                    | StatementKind::StorageDead(_) => true,

                    StatementKind::Assign(_) => {
                        dead_set.contains(&Location {
                            block: bb,
                            statement_index,
                        })
                    }

                    _ => false,
                };

                if !ok {
                    return false;
                }
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
        let borrowed_locals = borrowed_locals(body);
        let debuginfo_locals = debuginfo_locals(body);
  
        // phase 2
        let analysis = MaybeTransitiveLiveLocals::new(&borrowed_locals, &debuginfo_locals);

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

                live.seek_before_primary_effect(loc);

                if !live.get().contains(local) {
                    dead_locations.push(loc);
                }
            }
        }

        let dead_set: FxHashSet<Location> = dead_locations.iter().copied().collect();
        
        // phase 1
        let post_dom = crate::post_dom::PostDomGraph::new(body);
        let doms = rustc_data_structures::graph::dominators::dominators(&post_dom);

        // this collapses branches whose control dependent blocks contain only  statements already proven dead by the liveness analysis.
        for bb in body.basic_blocks.indices() {
            let successors: Vec<_> = body.basic_blocks[bb].terminator().successors().collect();

            if successors.len() > 1 && branch_is_dead(body, bb, &doms, &dead_set) {
                let Some(merge) = doms.immediate_dominator(bb) else {
                    continue;
                };

                if body.basic_blocks[bb].is_cleanup != body.basic_blocks[merge].is_cleanup {
                    continue;
                }

                body.basic_blocks_mut()[bb].terminator_mut().kind =
                    TerminatorKind::Goto { target: merge };
            }
        }

        for loc in dead_locations {
            body.basic_blocks_mut()[loc.block]
                .statements[loc.statement_index]
                .make_nop(true);
        }

        SimplifyCfg::Final.run_pass(tcx, body);
    }
}