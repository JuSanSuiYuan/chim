use chim_ir::{IRModule, IRFunction, BasicBlock, BlockId, IRInst, Terminator};
use chim_semantic::TypeId;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, PartialEq)]
pub struct ControlFlowGraph {
    pub function_id: usize,
    pub blocks: Vec<CFGBlock>,
    pub entry: BlockId,
    pub exit: Option<BlockId>,
    pub predecessors: HashMap<BlockId, Vec<BlockId>>,
    pub successors: HashMap<BlockId, Vec<BlockId>>,
    pub dom_tree: DominatorTree,
    pub post_dom_tree: PostDominatorTree,
    pub loops: Vec<NaturalLoop>,
    pub dataflow: DataFlowResults,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CFGBlock {
    pub id: BlockId,
    pub label: String,
    pub instructions: Vec<IRInst>,
    pub terminator: Terminator,
    pub phis: Vec<PhiNode>,
    pub is_entry: bool,
    pub is_exit: bool,
    pub is_loop_header: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhiNode {
    pub result: TypeId,
    pub incoming: Vec<(BlockId, TypeId)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DominatorTree {
    pub tree: HashMap<BlockId, Vec<BlockId>>,
    pub idom: HashMap<BlockId, BlockId>,
    pub depth: HashMap<BlockId, usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostDominatorTree {
    pub tree: HashMap<BlockId, Vec<BlockId>>,
    pub idom: HashMap<BlockId, BlockId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NaturalLoop {
    pub header: BlockId,
    pub blocks: HashSet<BlockId>,
    pub preheader: Option<BlockId>,
    pub exits: Vec<BlockId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataFlowResults {
    pub reaching_defs: HashMap<BlockId, Vec<TypeId>>,
    pub live_vars: HashMap<BlockId, HashSet<TypeId>>,
    pub available_exprs: HashMap<BlockId, HashSet<TypeId>>,
    pub very_busy_exprs: HashMap<BlockId, HashSet<TypeId>>,
}

impl ControlFlowGraph {
    pub fn build_from_function(module: &IRModule, func: &IRFunction) -> Self {
        let mut cfg = ControlFlowGraph {
            function_id: func.id.0,
            blocks: Vec::new(),
            entry: BlockId(0),
            exit: None,
            predecessors: HashMap::new(),
            successors: HashMap::new(),
            dom_tree: DominatorTree::new(),
            post_dom_tree: PostDominatorTree::new(),
            loops: Vec::new(),
            dataflow: DataFlowResults::new(),
        };

        for block in &func.body {
            cfg.blocks.push(CFGBlock {
                id: block.id,
                label: format!(".L{}", block.id.0),
                instructions: block.instructions.clone(),
                terminator: block.terminator.clone(),
                phis: Vec::new(),
                is_entry: block.id.0 == 0,
                is_exit: matches!(block.terminator, Terminator::Return(_)),
                is_loop_header: false,
            });

            cfg.add_edges(block);
        }

        cfg.exit = cfg.blocks.iter()
            .find(|b| b.is_exit)
            .map(|b| b.id);

        cfg.compute_dominators();
        cfg.compute_post_dominators();
        cfg.find_natural_loops();
        cfg.compute_dataflow();

        cfg
    }

    fn add_edges(&mut self, block: &BasicBlock) {
        let successors: Vec<BlockId> = match &block.terminator {
            Terminator::Branch(target) => vec![*target],
            Terminator::ConditionalBranch { true_block, false_block, .. } => vec![*true_block, *false_block],
            Terminator::Return(_) | Terminator::Unreachable => Vec::new(),
            Terminator::Switch { default_block, cases, .. } => {
                let mut succs = vec![*default_block];
                for (_, target) in cases {
                    succs.push(*target);
                }
                succs
            }
            Terminator::Invoke { normal_block, unwind_block, .. } => vec![*normal_block, *unwind_block],
        };

        self.successors.insert(block.id, successors.clone());

        for &target in &successors {
            self.predecessors.entry(target).or_default().push(block.id);
        }
    }

    fn compute_dominators(&mut self) {
        let all_blocks: HashSet<BlockId> = self.blocks.iter().map(|b| b.id).collect();
        let mut dom: HashMap<BlockId, HashSet<BlockId>> = HashMap::new();

        for &block_id in &all_blocks {
            dom.insert(block_id, all_blocks.clone());
        }

        dom.insert(self.entry, HashSet::from([self.entry]));

        let mut changed = true;
        while changed {
            changed = false;
            for &block_id in &all_blocks {
                if block_id == self.entry {
                    continue;
                }

                let mut new_dom = HashSet::from([block_id]);
                if let Some(preds) = self.predecessors.get(&block_id) {
                    if !preds.is_empty() {
                        let mut intersection = all_blocks.clone();
                        for &pred in preds {
                            intersection = intersection.intersection(&dom[&pred]).cloned().collect();
                        }
                        new_dom.extend(intersection);
                    }
                }

                if new_dom != dom[&block_id] {
                    dom.insert(block_id, new_dom);
                    changed = true;
                }
            }
        }

        let mut idom: HashMap<BlockId, BlockId> = HashMap::new();
        for &block_id in &all_blocks {
            if block_id == self.entry {
                continue;
            }

            let dom_set = &dom[&block_id];
            let mut candidates: HashSet<BlockId> = dom_set.iter().filter(|&&b| b != block_id).cloned().collect();

            if let Some(preds) = self.predecessors.get(&block_id) {
                for &pred in preds {
                    let pred_dom = &dom[&pred];
                    candidates.retain(|b| pred_dom.contains(b));
                }
            }

            if let Some(&immediate) = candidates.iter().next() {
                idom.insert(block_id, immediate);
            }
        }

        let mut depth: HashMap<BlockId, usize> = HashMap::new();
        depth.insert(self.entry, 0);

        for &block_id in &all_blocks {
            if block_id == self.entry {
                continue;
            }
            if let Some(&parent) = idom.get(&block_id) {
                depth.insert(block_id, depth[&parent] + 1);
            }
        }

        let mut tree: HashMap<BlockId, Vec<BlockId>> = HashMap::new();
        for (&child, &parent) in &idom {
            tree.entry(parent).or_default().push(child);
        }

        self.dom_tree = DominatorTree { tree, idom, depth };
    }

    fn compute_post_dominators(&mut self) {
        let all_blocks: HashSet<BlockId> = self.blocks.iter().map(|b| b.id).collect();

        let mut post_dom: HashMap<BlockId, HashSet<BlockId>> = HashMap::new();

        for &block_id in &all_blocks {
            post_dom.insert(block_id, all_blocks.clone());
        }

        if let Some(exit) = self.exit {
            post_dom.insert(exit, HashSet::from([exit]));
        }

        let mut changed = true;
        while changed {
            changed = false;
            for &block_id in &all_blocks {
                if Some(block_id) == self.exit {
                    continue;
                }

                let successors = self.successors.get(&block_id).cloned().unwrap_or_default();
                if successors.is_empty() {
                    continue;
                }

                let mut new_post_dom = HashSet::from([block_id]);
                let mut first = true;
                for &succ in &successors {
                    let succ_post = &post_dom[&succ];
                    if first {
                        new_post_dom = succ_post.clone();
                        first = false;
                    } else {
                        new_post_dom = new_post_dom.intersection(succ_post).cloned().collect();
                    }
                }
                new_post_dom.insert(block_id);

                if new_post_dom != post_dom[&block_id] {
                    post_dom.insert(block_id, new_post_dom);
                    changed = true;
                }
            }
        }

        let mut idom: HashMap<BlockId, BlockId> = HashMap::new();

        for &block_id in &all_blocks {
            if Some(block_id) == self.exit {
                continue;
            }

            let post_dom_set = &post_dom[&block_id];
            let mut candidates: HashSet<BlockId> = post_dom_set.iter().filter(|&&b| b != block_id).cloned().collect();

            let successors = self.successors.get(&block_id).cloned().unwrap_or_default();
            for &succ in &successors {
                let succ_post = &post_dom[&succ];
                candidates.retain(|b| succ_post.contains(b));
            }

            if let Some(&immediate) = candidates.iter().next() {
                idom.insert(block_id, immediate);
            }
        }

        let mut tree: HashMap<BlockId, Vec<BlockId>> = HashMap::new();
        for (&child, &parent) in &idom {
            tree.entry(parent).or_default().push(child);
        }

        self.post_dom_tree = PostDominatorTree { tree, idom };
    }

    fn find_natural_loops(&mut self) {
        for &block_id in self.blocks.iter().map(|b| b.id) {
            if let Some(preds) = self.predecessors.get(&block_id) {
                let mut loop_blocks = HashSet::new();
                loop_blocks.insert(block_id);

                let mut stack = Vec::new();
                for &pred in preds {
                    if pred != block_id && self.is_dominated_by(pred, block_id) {
                        stack.push(pred);
                        loop_blocks.insert(pred);
                    }
                }

                while let Some(current) = stack.pop() {
                    if let Some(current_preds) = self.predecessors.get(&current) {
                        for &pred in current_preds {
                            if !loop_blocks.contains(&pred) && self.is_dominated_by(pred, block_id) {
                                loop_blocks.insert(pred);
                                stack.push(pred);
                            }
                        }
                    }
                }

                if loop_blocks.len() > 1 || loop_blocks.contains(&block_id) {
                    let exits: Vec<BlockId> = loop_blocks.iter()
                        .filter(|&&b| {
                            if let Some(succs) = self.successors.get(&b) {
                                succs.iter().any(|&s| !loop_blocks.contains(&s))
                            } else {
                                false
                            }
                        })
                        .cloned()
                        .collect();

                    if !exits.is_empty() || loop_blocks.len() > 1 {
                        self.loops.push(NaturalLoop {
                            header: block_id,
                            blocks: loop_blocks,
                            preheader: None,
                            exits,
                        });

                        if let Some(block) = self.blocks.iter_mut().find(|b| b.id == block_id) {
                            block.is_loop_header = true;
                        }
                    }
                }
            }
        }
    }

    fn is_dominated_by(&self, block: BlockId, dominator: BlockId) -> bool {
        let mut current = block;
        while current != dominator {
            if current == self.entry {
                return false;
            }
            if let Some(&parent) = self.dom_tree.idom.get(&current) {
                current = parent;
            } else {
                return false;
            }
        }
        true
    }

    fn compute_dataflow(&mut self) {
        self.compute_reaching_defs();
        self.compute_live_vars();
        self.compute_available_exprs();
        self.compute_very_busy_exprs();
    }

    fn compute_reaching_defs(&mut self) {
        let mut in_sets: HashMap<BlockId, HashSet<TypeId>> = HashMap::new();
        let mut out_sets: HashMap<BlockId, HashSet<TypeId>> = HashMap::new();

        for &block_id in self.blocks.iter().map(|b| b.id) {
            in_sets.insert(block_id, HashSet::new());
            out_sets.insert(block_id, HashSet::new());
        }

        let mut changed = true;
        while changed {
            changed = false;
            for &block_id in self.blocks.iter().map(|b| b.id) {
                let mut new_in = HashSet::new();
                if let Some(preds) = self.predecessors.get(&block_id) {
                    for &pred in preds {
                        new_in.extend(out_sets.get(&pred).cloned().unwrap_or_default());
                    }
                }

                if new_in != *in_sets.get(&block_id).unwrap_or(&HashSet::new()) {
                    in_sets.insert(block_id, new_in.clone());
                    changed = true;
                }

                let mut new_out = new_in;
                for inst in &self.blocks.iter().find(|b| b.id == block_id).map(|b| &b.instructions).unwrap_or(&Vec::new()) {
                    if let chim_ir::IRInst::Alloca { ty, .. } = inst {
                        new_out.insert(*ty);
                    }
                }

                if new_out != *out_sets.get(&block_id).unwrap_or(&HashSet::new()) {
                    out_sets.insert(block_id, new_out);
                    changed = true;
                }
            }
        }

        self.dataflow.reaching_defs = in_sets;
    }

    fn compute_live_vars(&mut self) {
        let mut use_sets: HashMap<BlockId, HashSet<TypeId>> = HashMap::new();
        let mut def_sets: HashMap<BlockId, HashSet<TypeId>> = HashMap::new();

        for &block_id in self.blocks.iter().map(|b| b.id) {
            let mut use_set = HashSet::new();
            let mut def_set = HashSet::new();

            for inst in &self.blocks.iter().find(|b| b.id == block_id).map(|b| &b.instructions).unwrap_or(&Vec::new()) {
                match inst {
                    chim_ir::IRInst::Load { src, .. } => {
                        use_set.insert(*src);
                    }
                    chim_ir::IRInst::Store { dest, .. } => {
                        def_set.insert(*dest);
                    }
                    _ => {}
                }
            }

            use_sets.insert(block_id, use_set);
            def_sets.insert(block_id, def_set);
        }

        let mut in_sets: HashMap<BlockId, HashSet<TypeId>> = HashMap::new();
        let mut out_sets: HashMap<BlockId, HashSet<TypeId>> = HashMap::new();

        for &block_id in self.blocks.iter().map(|b| b.id) {
            in_sets.insert(block_id, HashSet::new());
            out_sets.insert(block_id, HashSet::new());
        }

        let mut changed = true;
        while changed {
            changed = false;
            for &block_id in self.blocks.iter().map(|b| b.id).rev() {
                let mut new_out = HashSet::new();
                if let Some(succs) = self.successors.get(&block_id) {
                    for &succ in succs {
                        new_out.extend(in_sets.get(&succ).cloned().unwrap_or_default());
                    }
                }

                let use_set = use_sets.get(&block_id).cloned().unwrap_or_default();
                let def_set = def_sets.get(&block_id).cloned().unwrap_or_default();

                let mut new_in = use_set.clone();
                for &var in &new_out {
                    if !def_set.contains(&var) {
                        new_in.insert(var);
                    }
                }

                if new_in != *in_sets.get(&block_id).unwrap_or(&HashSet::new()) {
                    in_sets.insert(block_id, new_in.clone());
                    changed = true;
                }

                if new_out != *out_sets.get(&block_id).unwrap_or(&HashSet::new()) {
                    out_sets.insert(block_id, new_out);
                    changed = true;
                }
            }
        }

        self.dataflow.live_vars = in_sets;
    }

    fn compute_available_exprs(&mut self) {
        let mut expr_sets: HashMap<BlockId, HashSet<String>> = HashMap::new();

        for &block_id in self.blocks.iter().map(|b| b.id) {
            expr_sets.insert(block_id, HashSet::new());
        }

        expr_sets.insert(self.entry, HashSet::new());

        let mut changed = true;
        while changed {
            changed = false;
            for &block_id in self.blocks.iter().map(|b| b.id) {
                let mut new_in = HashSet::new();
                if let Some(preds) = self.predecessors.get(&block_id) {
                    for &pred in preds {
                        new_in.extend(expr_sets.get(&pred).cloned().unwrap_or_default());
                    }
                }

                let mut new_out = new_in.clone();
                for inst in &self.blocks.iter().find(|b| b.id == block_id).map(|b| &b.instructions).unwrap_or(&Vec::new()) {
                    match inst {
                        chim_ir::IRInst::Binary { left, right, op, .. } => {
                            new_out.insert(format!("{:?} {:?} {:?}", left, op, right));
                        }
                        _ => {}
                    }
                }

                if new_out != *expr_sets.get(&block_id).unwrap_or(&HashSet::new()) {
                    expr_sets.insert(block_id, new_out);
                    changed = true;
                }
            }
        }

        self.dataflow.available_exprs = expr_sets
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().map(|s| s.parse().unwrap_or(0)).collect()))
            .collect();
    }

    fn compute_very_busy_exprs(&mut self) {
        let mut expr_sets: HashMap<BlockId, HashSet<String>> = HashMap::new();

        for &block_id in self.blocks.iter().map(|b| b.id) {
            expr_sets.insert(block_id, HashSet::new());
        }

        let mut changed = true;
        while changed {
            changed = false;
            for &block_id in self.blocks.iter().map(|b| b.id).rev() {
                let mut new_out = HashSet::new();
                if let Some(succs) = self.successors.get(&block_id) {
                    for &succ in succs {
                        new_out.extend(expr_sets.get(&succ).cloned().unwrap_or_default());
                    }
                }

                let mut new_in = new_out.clone();
                for inst in &self.blocks.iter().find(|b| b.id == block_id).map(|b| &b.instructions).unwrap_or(&Vec::new()) {
                    match inst {
                        chim_ir::IRInst::Binary { left, right, op, .. } => {
                            if let Some(expr) = new_out.get(&format!("{:?} {:?} {:?}", left, op, right)) {
                                new_in.insert(expr.clone());
                            }
                        }
                        _ => {}
                    }
                }

                if new_in != *expr_sets.get(&block_id).unwrap_or(&HashSet::new()) {
                    expr_sets.insert(block_id, new_in);
                    changed = true;
                }
            }
        }

        self.dataflow.very_busy_exprs = expr_sets
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().map(|s| s.parse().unwrap_or(0)).collect()))
            .collect();
    }

    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    pub fn edge_count(&self) -> usize {
        self.successors.values().map(|v| v.len()).sum()
    }

    pub fn has_loop(&self, block_id: BlockId) -> bool {
        self.loops.iter().any(|l| l.header == block_id)
    }

    pub fn is_loop_header(&self, block_id: BlockId) -> bool {
        self.loops.iter().any(|l| l.header == block_id && l.blocks.contains(&block_id))
    }
}

impl DominatorTree {
    fn new() -> Self {
        DominatorTree {
            tree: HashMap::new(),
            idom: HashMap::new(),
            depth: HashMap::new(),
        }
    }

    pub fn immediate_dominator(&self, block: BlockId) -> Option<BlockId> {
        self.idom.get(&block).copied()
    }

    pub fn dominators(&self, block: BlockId) -> Vec<BlockId> {
        let mut dominators = Vec::new();
        let mut current = block;
        while let Some(&idom) = self.idom.get(&current) {
            dominators.push(idom);
            current = idom;
        }
        dominators
    }

    pub fn depth(&self, block: BlockId) -> usize {
        self.depth.get(&block).copied().unwrap_or(0)
    }
}

impl PostDominatorTree {
    fn new() -> Self {
        PostDominatorTree {
            tree: HashMap::new(),
            idom: HashMap::new(),
        }
    }
}

impl DataFlowResults {
    fn new() -> Self {
        DataFlowResults {
            reaching_defs: HashMap::new(),
            live_vars: HashMap::new(),
            available_exprs: HashMap::new(),
            very_busy_exprs: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfg_creation() {
        let mut module = chim_ir::IRModule {
            functions: Vec::new(),
            globals: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),
        };

        let func = chim_ir::IRFunction {
            id: chim_ir::FunctionId(0),
            name: "test".to_string(),
            params: Vec::new(),
            return_type: chim_semantic::TypeId(0),
            body: vec![
                chim_ir::BasicBlock {
                    id: chim_ir::BlockId(0),
                    instructions: Vec::new(),
                    terminator: chim_ir::Terminator::Branch(chim_ir::BlockId(1)),
                    predecessors: Vec::new(),
                    successors: vec![chim_ir::BlockId(1)],
                },
                chim_ir::BasicBlock {
                    id: chim_ir::BlockId(1),
                    instructions: Vec::new(),
                    terminator: chim_ir::Terminator::Ret(Some(chim_ir::ValueId(0))),
                    predecessors: vec![chim_ir::BlockId(0)],
                    successors: Vec::new(),
                },
            ],
            span: chim_span::Span::new(chim_span::FileId(0), 0, 0, 0, 0),
            is_pub: false,
            is_extern: false,
            is_unsafe: false,
        };

        let cfg = ControlFlowGraph::build_from_function(&module, &func);
        assert_eq!(cfg.block_count(), 2);
    }

    #[test]
    fn test_dominator_depth() {
        let tree = DominatorTree::new();
        assert_eq!(tree.depth(chim_ir::BlockId(0)), 0);
    }
}
