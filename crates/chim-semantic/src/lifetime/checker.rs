use std::collections::HashMap;

use crate::parser::{AstNode, BlockId, NodeId};
use crate::typechecker::{Lifetime, LifetimeAnnotation, Type, TypeId, VarId};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AllocationLifetime {
    Return,
    Param { var_id: VarId },
    Scope { level: usize },
    Unknown,
}

#[derive(Debug, Default)]
pub struct LifetimeDatabase {
    node_lifetimes: HashMap<NodeId, AllocationLifetime>,
    exiting_blocks: HashMap<NodeId, Vec<BlockId>>,
    possible_allocation_sites: Vec<(Vec<BlockId>, usize, NodeId)>,
    num_lifetime_inferences: HashMap<BlockId, usize>,
}

impl LifetimeDatabase {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_node_lifetime(&self, node_id: NodeId) -> AllocationLifetime {
        self.node_lifetimes.get(&node_id).copied().unwrap_or(AllocationLifetime::Unknown)
    }

    pub fn set_node_lifetime(&mut self, node_id: NodeId, lifetime: AllocationLifetime) {
        self.node_lifetimes.insert(node_id, lifetime);
    }

    pub fn resize_node_lifetimes(&mut self, size: usize, default: AllocationLifetime) {
        for i in 0..size {
            self.node_lifetimes.entry(NodeId(i)).or_insert(default);
        }
    }

    pub fn add_possible_allocation(&mut self, blocks: Vec<BlockId>, level: usize, node_id: NodeId) {
        self.possible_allocation_sites.push((blocks, level, node_id));
    }

    pub fn get_allocation_sites(&self) -> &[(Vec<BlockId>, usize, NodeId)] {
        &self.possible_allocation_sites
    }

    pub fn exiting_blocks(&self) -> &HashMap<NodeId, Vec<BlockId>> {
        &self.exiting_blocks
    }

    pub fn exiting_blocks_mut(&mut self) -> &mut HashMap<NodeId, Vec<BlockId>> {
        &mut self.exiting_blocks
    }

    pub fn increment_inference(&mut self, block_id: BlockId) {
        *self.num_lifetime_inferences.entry(block_id).or_insert(0) += 1;
    }

    pub fn get_inference_count(&self, block_id: BlockId) -> usize {
        self.num_lifetime_inferences.get(&block_id).copied().unwrap_or(0)
    }
}

pub struct LifetimeChecker<'a, 'b> {
    compiler: &'a mut crate::Compiler,
    db: &'a mut LifetimeDatabase,
    current_blocks: Vec<BlockId>,
    scope_level: usize,
}

impl<'a, 'b> LifetimeChecker<'a, 'b> {
    pub fn new(compiler: &'a mut crate::Compiler, db: &'a mut LifetimeDatabase) -> Self {
        Self {
            compiler,
            db,
            current_blocks: Vec::new(),
            scope_level: 0,
        }
    }

    pub fn check(mut self) -> LifetimeDatabase {
        let num_nodes = self.compiler.num_ast_nodes();
        self.db.resize_node_lifetimes(num_nodes, AllocationLifetime::Unknown);
        self.initialize_param_lifetimes();
        self.check_function_bodies();
        self.analyze_allocation_sites();
        self.db.clone()
    }

    fn initialize_param_lifetimes(&mut self) {
        for (fun_id, fun) in self.compiler.functions.iter().enumerate().skip(1) {
            for param in &fun.params {
                let param_node_id = self.compiler.get_variable(param.var_id).where_defined;
                
                for annotation in &fun.lifetime_annotations {
                    match annotation {
                        LifetimeAnnotation::Equality(Lifetime::Return, Lifetime::Variable(vid)) 
                            if vid == &param.var_id => {
                            self.db.set_node_lifetime(param_node_id, AllocationLifetime::Return);
                            continue;
                        }
                        LifetimeAnnotation::Equality(Lifetime::Variable(vid), Lifetime::Return) 
                            if vid == &param.var_id => {
                            self.db.set_node_lifetime(param_node_id, AllocationLifetime::Return);
                            continue;
                        }
                        LifetimeAnnotation::Equality(Lifetime::Variable(lhs), Lifetime::Variable(rhs)) 
                            if lhs == &param.var_id => {
                            self.db.set_node_lifetime(param_node_id, AllocationLifetime::Param { var_id: *rhs });
                            continue;
                        }
                        _ => {}
                    }
                }

                self.db.set_node_lifetime(
                    param_node_id, 
                    AllocationLifetime::Param { var_id: param.var_id }
                );
            }
        }
    }

    fn check_function_bodies(&mut self) {
        for (fun_id, fun) in self.compiler.functions.iter().enumerate().skip(1) {
            if let Some(body) = fun.body {
                self.current_blocks.clear();
                self.scope_level = 0;
                self.check_node(body);
            }
        }
    }

    fn check_node(&mut self, node_id: NodeId) {
        match self.compiler.get_node(node_id) {
            AstNode::Block(block_id) => {
                self.current_blocks.push(*block_id);
                let inner_level = self.scope_level;
                self.scope_level += 1;
                
                for node in &self.compiler.blocks[block_id.0].nodes {
                    self.check_node(*node);
                }
                
                self.scope_level = inner_level;
                self.current_blocks.pop();
            }
            AstNode::UnsafeBlock(block) => {
                let saved_level = self.scope_level;
                self.scope_level = 0;
                self.check_node(*block);
                self.scope_level = saved_level;
            }
            AstNode::Int | AstNode::Float | AstNode::True | AstNode::False | 
            AstNode::String | AstNode::CString | AstNode::CChar | AstNode::None => {}
            
            AstNode::Let { initializer, .. } => {
                if let Some(init) = initializer {
                    self.expand_lifetime(node_id, node_id, AllocationLifetime::Scope { level: self.scope_level });
                    self.expand_lifetime_with_node(*init, node_id);
                    self.check_node(*init);
                    self.expand_lifetime_with_node(node_id, *init);
                }
            }
            
            AstNode::Name => {
                if let Some(var_id) = self.compiler.var_resolution.get(&node_id) {
                    let def_node_id = self.compiler.get_variable(*var_id).where_defined;
                    self.expand_lifetime_with_node(def_node_id, node_id);
                    self.expand_lifetime_with_node(node_id, def_node_id);
                }
            }
            
            AstNode::BinaryOp { lhs, op, rhs } => {
                if matches!(self.compiler.get_node(*op), AstNode::Assignment) {
                    self.check_lvalue_lifetime(*lhs);
                    if matches!(self.db.get_node_lifetime(*lhs), AllocationLifetime::Unknown) {
                        self.expand_lifetime(*lhs, *lhs, AllocationLifetime::Scope { level: self.scope_level });
                    }
                    self.check_node(*rhs);
                    self.expand_lifetime_with_node(*rhs, *lhs);
                    self.expand_lifetime_with_node(*lhs, *rhs);
                } else {
                    self.expand_lifetime_with_node(*lhs, node_id);
                    self.expand_lifetime_with_node(*rhs, node_id);
                    self.check_node(*lhs);
                    self.check_node(*rhs);
                }
            }
            
            AstNode::Call { head, args } => {
                let call_target = self.compiler.call_resolution.get(head);
                if let Some(&crate::CallTarget::Function(fun_id)) = call_target {
                    if fun_id.0 != 0 {
                        self.check_node(*head);
                        let params = self.compiler.functions[fun_id.0].params.clone();
                        if self.compiler.functions[fun_id.0].body.is_some() {
                            for (param, arg) in params.iter().zip(args.iter()) {
                                let param_node_id = self.compiler.get_variable(param.var_id).where_defined;
                                let expected_lifetime = self.db.get_node_lifetime(param_node_id);
                                match expected_lifetime {
                                    AllocationLifetime::Return => {
                                        self.expand_lifetime_with_node(*arg, node_id);
                                        self.check_node(*arg);
                                    }
                                    AllocationLifetime::Param { var_id: pid } => {
                                        for (p, a) in params.iter().zip(args.iter()) {
                                            if p.var_id == pid && a != arg {
                                                self.expand_lifetime_with_node(*a, *arg);
                                                break;
                                            }
                                        }
                                        self.check_node(*arg);
                                    }
                                    _ => self.check_node(*arg),
                                }
                            }
                        }
                    } else {
                        self.check_node(*head);
                        for arg in args { self.check_node(*arg); }
                    }
                }
                self.db.add_possible_allocation(self.current_blocks.clone(), self.scope_level, node_id);
            }
            
            AstNode::Return(return_expr) => {
                self.db.exiting_blocks_mut().insert(node_id, self.current_blocks.clone());
                if let Some(expr) = return_expr {
                    self.expand_lifetime(*expr, *expr, AllocationLifetime::Return);
                    self.check_node(*expr);
                }
            }
            
            AstNode::Break => {
                self.db.exiting_blocks_mut().insert(
                    node_id,
                    vec![*self.current_blocks.last().expect("missing current block")]
                );
            }
            
            AstNode::If { condition, then_block, else_expression } => {
                self.expand_lifetime_with_node(*condition, node_id);
                self.expand_lifetime_with_node(*then_block, node_id);
                self.check_node(*condition);
                self.check_node(*then_block);
                if let Some(else_expr) = else_expression {
                    self.expand_lifetime_with_node(*else_expr, node_id);
                    self.check_node(*else_expr);
                }
            }
            
            AstNode::While { condition, block } => {
                self.expand_lifetime_with_node(*condition, node_id);
                self.expand_lifetime_with_node(*block, node_id);
                self.check_node(*condition);
                self.check_node(*block);
            }
            
            AstNode::ResizeRawBuffer { pointer, .. } | AstNode::Defer { pointer, .. } => {
                self.check_node(*pointer);
                let lifetime = self.db.get_node_lifetime(*pointer);
                if matches!(lifetime, AllocationLifetime::Scope { .. }) {
                    self.db.add_possible_allocation(self.current_blocks.clone(), self.scope_level, node_id);
                }
            }
            
            AstNode::New(_, required_lifetime, allocation_node_id) => {
                let alloc_node = *allocation_node_id;
                let req_lifetime = *required_lifetime;
                
                if self.db.get_node_lifetime(node_id) == AllocationLifetime::Unknown {
                    self.expand_lifetime(alloc_node, node_id, AllocationLifetime::Scope { level: self.scope_level });
                    self.expand_lifetime(node_id, node_id, AllocationLifetime::Scope { level: self.scope_level });
                } else {
                    self.expand_lifetime_with_node(alloc_node, node_id);
                }
                
                match self.compiler.get_node(alloc_node) {
                    AstNode::Call { args, .. } => {
                        for arg in args {
                            self.check_node(*arg);
                            self.expand_lifetime_with_node(*arg, node_id);
                            self.expand_lifetime_with_node(alloc_node, *arg);
                            self.expand_lifetime_with_node(node_id, *arg);
                        }
                    }
                    _ => {}
                }
                
                if matches!(req_lifetime, crate::RequiredLifetime::Local) {
                    if !matches!(self.db.get_node_lifetime(node_id), AllocationLifetime::Scope { .. }) {
                        self.compiler.errors.push(crate::SourceError {
                            message: format!("allocation is not local, lifetime inferred to be {:?}", 
                                self.db.get_node_lifetime(node_id)),
                            node_id,
                            severity: crate::Severity::Error,
                        });
                    }
                }
                
                self.db.add_possible_allocation(self.current_blocks.clone(), self.scope_level, node_id);
            }
            
            AstNode::RawBuffer(items) => {
                self.expand_lifetime(node_id, node_id, AllocationLifetime::Scope { level: self.scope_level });
                for item in items {
                    self.expand_lifetime_with_node(*item, node_id);
                    self.check_node(*item);
                }
                self.db.add_possible_allocation(self.current_blocks.clone(), self.scope_level, node_id);
            }
            
            AstNode::MemberAccess { target, .. } => {
                let tgt = *target;
                let node_type = self.compiler.get_node_type(node_id);
                if !self.compiler.is_copyable_type(node_type) {
                    self.expand_lifetime_with_node(tgt, node_id);
                    self.expand_lifetime_with_node(node_id, tgt);
                }
                self.check_node(tgt);
            }
            
            AstNode::Match { target, match_arms } => {
                self.expand_lifetime_with_node(*target, node_id);
                for (_, result) in match_arms {
                    self.check_node(*result);
                }
            }
            
            AstNode::Block(..) | AstNode::Statement(..) => {
                self.check_node(match self.compiler.get_node(node_id) {
                    AstNode::Statement(s) => *s,
                    AstNode::Block(b) => *b,
                    _ => unreachable!(),
                });
            }
            
            AstNode::Type { .. } | AstNode::TypeCoercion { .. } => {}
            
            AstNode::Fun { .. } | AstNode::Struct { .. } | AstNode::Enum { .. } | 
            AstNode::ExternType { .. } => {}
            
            AstNode::NamedValue { value, .. } => {
                let val = *value;
                self.expand_lifetime_with_node(val, node_id);
                self.check_node(val);
                self.expand_lifetime_with_node(node_id, val);
            }
            
            AstNode::Index { target, .. } => {
                let tgt = *target;
                self.expand_lifetime_with_node(tgt, node_id);
                self.check_node(tgt);
            }
            
            AstNode::For { block, .. } => {
                self.expand_lifetime_with_node(*block, node_id);
                self.check_node(*block);
            }
            
            _ => {}
        }
    }
    
    fn check_lvalue_lifetime(&mut self, lvalue: NodeId) {
        match self.compiler.get_node(lvalue) {
            AstNode::Name => {
                if let Some(var_id) = self.compiler.var_resolution.get(&lvalue) {
                    let def_node_id = self.compiler.get_variable(*var_id).where_defined;
                    self.expand_lifetime_with_node(lvalue, def_node_id);
                }
            }
            AstNode::Index { target, .. } => {
                let tgt = *target;
                self.check_lvalue_lifetime(tgt);
                self.expand_lifetime_with_node(lvalue, tgt);
            }
            AstNode::MemberAccess { target, .. } => {
                let tgt = *target;
                self.check_lvalue_lifetime(tgt);
                self.expand_lifetime_with_node(lvalue, tgt);
            }
            _ => {}
        }
    }
    
    fn expand_lifetime_with_node(&mut self, node_id: NodeId, lifetime_from_node: NodeId) {
        let lifetime = self.db.get_node_lifetime(lifetime_from_node);
        self.expand_lifetime(node_id, lifetime_from_node, lifetime);
    }
    
    fn expand_lifetime(&mut self, node_id: NodeId, _source: NodeId, lifetime: AllocationLifetime) {
        if lifetime == AllocationLifetime::Unknown {
            return;
        }
        
        let current = self.db.get_node_lifetime(node_id);
        match current {
            AllocationLifetime::Unknown => {
                self.db.set_node_lifetime(node_id, lifetime);
                self.increment_inference_counter();
            }
            AllocationLifetime::Param { var_id: curr_var } => {
                if let AllocationLifetime::Param { var_id: new_var } = lifetime {
                    if new_var != curr_var {
                        self.compiler.errors.push(crate::SourceError {
                            message: format!("can't find compatible lifetime between params"),
                            node_id,
                            severity: crate::Severity::Error,
                        });
                    }
                }
            }
            AllocationLifetime::Return => {
                if let AllocationLifetime::Param { .. } = lifetime {
                    self.compiler.errors.push(crate::SourceError {
                        message: format!("can't find compatible lifetime for return"),
                        node_id,
                        severity: crate::Severity::Error,
                    });
                }
            }
            AllocationLifetime::Scope { level: curr_level } => {
                if let AllocationLifetime::Scope { level: new_level } = lifetime {
                    if new_level < curr_level {
                        self.db.set_node_lifetime(node_id, lifetime);
                        self.increment_inference_counter();
                    }
                } else if let AllocationLifetime::Param { var_id } = lifetime {
                    let var_type = self.compiler.get_variable(var_id).ty;
                    if self.compiler.is_allocator_type(var_type) {
                        self.db.set_node_lifetime(node_id, lifetime);
                        self.increment_inference_counter();
                    } else {
                        let var_name = self.compiler.get_variable(var_id).name;
                        self.compiler.errors.push(crate::SourceError {
                            message: format!("param '{}' is not an allocator", 
                                String::from_utf8_lossy(var_name)),
                            node_id,
                            severity: crate::Severity::Error,
                        });
                    }
                } else if lifetime == AllocationLifetime::Return {
                    self.db.set_node_lifetime(node_id, lifetime);
                    self.increment_inference_counter();
                }
            }
        }
    }
    
    fn increment_inference_counter(&mut self) {
        if let Some(block_id) = self.current_blocks.last() {
            self.db.increment_inference(*block_id);
        }
    }
    
    fn analyze_allocation_sites(&mut self) {
        for (block_ids, scope_level, node_id) in self.db.get_allocation_sites().to_vec() {
            if let AllocationLifetime::Scope { level } = self.db.get_node_lifetime(node_id) {
                if let Some(block_id) = block_ids.iter().rev().nth(scope_level - level) {
                    self.compiler.blocks[block_id.0].may_locally_allocate = Some(level);
                }
            }
        }
    }
}

pub fn check_lifetimes(compiler: &mut crate::Compiler) -> LifetimeDatabase {
    let mut db = LifetimeDatabase::new();
    let checker = LifetimeChecker::new(compiler, &mut db);
    checker.check()
}
