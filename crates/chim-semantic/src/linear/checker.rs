use std::collections::{HashMap, HashSet};

use crate::parser::{AstNode, NodeId};
use crate::Compiler;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinearKind {
    Linear,
    Consumed,
    Used,
}

#[derive(Debug, Default)]
pub struct LinearTypeDatabase {
    linear_values: HashMap<NodeId, LinearKind>,
    used_nodes: HashSet<NodeId>,
    consumed_nodes: HashSet<NodeId>,
    errors: Vec<LinearError>,
}

impl LinearTypeDatabase {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mark_linear(&mut self, node_id: NodeId) {
        self.linear_values.insert(node_id, LinearKind::Linear);
    }

    pub fn mark_used(&mut self, node_id: NodeId) {
        self.used_nodes.insert(node_id);
        if let Some(kind) = self.linear_values.get_mut(&node_id) {
            *kind = LinearKind::Used;
        }
    }

    pub fn mark_consumed(&mut self, node_id: NodeId) {
        self.consumed_nodes.insert(node_id);
        if let Some(kind) = self.linear_values.get_mut(&node_id) {
            *kind = LinearKind::Consumed;
        }
    }

    pub fn is_linear(&self, node_id: NodeId) -> bool {
        matches!(self.linear_values.get(&node_id), Some(LinearKind::Linear))
    }

    pub fn is_used(&self, node_id: NodeId) -> bool {
        self.used_nodes.contains(&node_id)
    }

    pub fn is_consumed(&self, node_id: NodeId) -> bool {
        self.consumed_nodes.contains(&node_id)
    }

    pub fn add_error(&mut self, error: LinearError) {
        self.errors.push(error);
    }

    pub fn errors(&self) -> &[LinearError] {
        &self.errors
    }
}

#[derive(Debug, Clone)]
pub struct LinearError {
    pub message: String,
    pub node_id: NodeId,
}

pub struct LinearTypeChecker<'a> {
    compiler: &'a mut Compiler,
    db: &'a mut LinearTypeDatabase,
}

impl<'a> LinearTypeChecker<'a> {
    pub fn new(compiler: &'a mut Compiler, db: &'a mut LinearTypeDatabase) -> Self {
        Self { compiler, db }
    }

    pub fn check(mut self) -> LinearTypeDatabase {
        self.collect_linear_types();
        self.check_consumed_once();
        self.db.clone()
    }

    fn collect_linear_types(&mut self) {
        for (type_id, ty) in self.compiler.types.iter().enumerate() {
            if self.is_linear_type(*ty) {
                let type_id = crate::TypeId(type_id);
                self.compiler.structs_with_linear.insert(type_id, true);
            }
        }
    }

    fn is_linear_type(&self, ty: &crate::Type) -> bool {
        match ty {
            crate::Type::Struct { is_linear, .. } => *is_linear,
            crate::Type::Enum { variants, .. } => {
                variants.iter().any(|v| matches!(v, crate::EnumVariant::Struct { .. }))
            }
            _ => false,
        }
    }

    fn check_consumed_once(&mut self) {
        for (fun_id, fun) in self.compiler.functions.iter().enumerate().skip(1) {
            if let Some(body) = fun.body {
                self.check_node(body);
            }
        }
    }

    fn check_node(&mut self, node_id: NodeId) {
        match self.compiler.get_node(node_id) {
            AstNode::Name => {
                if let Some(var_id) = self.compiler.var_resolution.get(&node_id) {
                    let var = self.compiler.get_variable(*var_id);
                    let var_type = var.ty;
                    if self.is_linear_type(self.compiler.get_type(var_type)) {
                        if self.db.is_used(node_id) {
                            self.db.add_error(LinearError {
                                message: format!(
                                    "linear value '{}' used more than once",
                                    String::from_utf8_lossy(var.name)
                                ),
                                node_id,
                            });
                        } else {
                            self.db.mark_used(node_id);
                        }
                    }
                }
            }

            AstNode::BinaryOp { lhs, op: _, rhs } => {
                let lhs_type = self.compiler.get_node_type(*lhs);
                let rhs_type = self.compiler.get_node_type(*rhs);
                
                if self.is_linear_type(self.compiler.get_type(lhs_type)) {
                    self.check_linear_use(*lhs);
                }
                if self.is_linear_type(self.compiler.get_type(rhs_type)) {
                    self.check_linear_use(*rhs);
                }
                
                self.check_node(*lhs);
                self.check_node(*rhs);
            }

            AstNode::Call { head, args } => {
                let call_target = self.compiler.call_resolution.get(head);
                if let Some(&crate::CallTarget::Function(fun_id)) = call_target {
                    let fun = &self.compiler.functions[fun_id.0];
                    
                    for (param, arg) in fun.params.iter().zip(args.iter()) {
                        let param_var = self.compiler.get_variable(param.var_id);
                        if self.is_linear_type(self.compiler.get_type(param_var.ty)) {
                            self.check_linear_use(*arg);
                        }
                    }
                }

                self.check_node(*head);
                for arg in args {
                    self.check_node(*arg);
                }
            }

            AstNode::Let { initializer, .. } => {
                if let Some(init) = initializer {
                    let init_type = self.compiler.get_node_type(*init);
                    if self.is_linear_type(self.compiler.get_type(init_type)) {
                        self.check_linear_use(*init);
                    }
                    self.check_node(*init);
                }
            }

            AstNode::If { condition, then_block, else_expression } => {
                let cond_type = self.compiler.get_node_type(*condition);
                if self.is_linear_type(self.compiler.get_type(cond_type)) {
                    self.check_linear_use(*condition);
                }
                self.check_node(*condition);
                self.check_node(*then_block);
                if let Some(else_expr) = else_expression {
                    self.check_node(*else_expr);
                }
            }

            AstNode::While { condition, block } => {
                let cond_type = self.compiler.get_node_type(*condition);
                if self.is_linear_type(self.compiler.get_type(cond_type)) {
                    self.check_linear_use(*condition);
                }
                self.check_node(*condition);
                self.check_node(*block);
            }

            AstNode::Return(return_expr) => {
                if let Some(expr) = return_expr {
                    let expr_type = self.compiler.get_node_type(*expr);
                    if self.is_linear_type(self.compiler.get_type(expr_type)) {
                        self.check_linear_use(*expr);
                    }
                    self.check_node(*expr);
                }
            }

            AstNode::Match { target, match_arms } => {
                let target_type = self.compiler.get_node_type(*target);
                if self.is_linear_type(self.compiler.get_type(target_type)) {
                    self.check_linear_use(*target);
                }
                self.check_node(*target);
                for (_, result) in match_arms {
                    self.check_node(*result);
                }
            }

            AstNode::MemberAccess { target, .. } => {
                let target_type = self.compiler.get_node_type(*target);
                if self.is_linear_type(self.compiler.get_type(target_type)) {
                    self.check_linear_use(*target);
                }
                self.check_node(*target);
            }

            AstNode::Block(block_id) => {
                for node in &self.compiler.blocks[block_id.0].nodes {
                    self.check_node(*node);
                }
            }

            AstNode::ResizeRawBuffer { pointer, .. } => {
                let ptr_type = self.compiler.get_node_type(*pointer);
                if self.is_linear_type(self.compiler.get_type(ptr_type)) {
                    self.check_linear_use(*pointer);
                }
                self.check_node(*pointer);
            }

            _ => {}
        }
    }

    fn check_linear_use(&mut self, node_id: NodeId) {
        if self.db.is_consumed(node_id) {
            let var_name = if let Some(var_id) = self.compiler.var_resolution.get(&node_id) {
                format!("'{}'", String::from_utf8_lossy(self.compiler.get_variable(*var_id).name))
            } else {
                "linear value".to_string()
            };
            
            self.db.add_error(LinearError {
                message: format!("{} used after consumption", var_name),
                node_id,
            });
        } else if self.db.is_used(node_id) {
            self.db.add_error(LinearError {
                message: "linear value used more than once".to_string(),
                node_id,
            });
        } else {
            self.db.mark_used(node_id);
        }
    }
}

pub fn check_linear_types(compiler: &mut Compiler) -> LinearTypeDatabase {
    let mut db = LinearTypeDatabase::new();
    let checker = LinearTypeChecker::new(compiler, &mut db);
    checker.check()
}
