use std::collections::HashMap;

use crate::parser::{AstNode, NodeId};
use crate::typechecker::{Type, TypeId};

#[derive(Debug, Clone)]
pub enum ComptimeValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Unit,
    Tuple(Vec<ComptimeValue>),
    Struct(HashMap<String, ComptimeValue>),
}

#[derive(Debug, Default)]
pub struct ComptimeEvaluator {
    values: HashMap<NodeId, ComptimeValue>,
    errors: Vec<ComptimeError>,
}

impl ComptimeEvaluator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn evaluate(&mut self, compiler: &crate::Compiler) {
        for (fun_id, fun) in compiler.functions.iter().enumerate().skip(1) {
            if fun.is_comptime && fun.body.is_some() {
                self.evaluate_function(fun, compiler);
            }
        }
    }

    fn evaluate_function(&mut self, fun: &crate::Function, compiler: &crate::Compiler) {
        if let Some(body) = fun.body {
            self.evaluate_node(body, compiler);
        }
    }

    fn evaluate_node(&mut self, node_id: NodeId, compiler: &crate::Compiler) -> Option<ComptimeValue> {
        match compiler.get_node(node_id) {
            AstNode::Int => {
                let src = compiler.get_source(node_id);
                src.parse::<i64>().ok().map(ComptimeValue::Int)
            }
            
            AstNode::Float => {
                let src = compiler.get_source(node_id);
                src.parse::<f64>().ok().map(ComptimeValue::Float)
            }
            
            AstNode::True => Some(ComptimeValue::Bool(true)),
            AstNode::False => Some(ComptimeValue::Bool(false)),
            AstNode::String => {
                let src = compiler.get_source(node_id);
                let content = &src[1..src.len()-1];
                Some(ComptimeValue::String(content.to_string()))
            }
            
            AstNode::Name => {
                self.values.get(&node_id).cloned()
            }
            
            AstNode::BinaryOp { lhs, op, rhs } => {
                let lhs_val = self.evaluate_node(*lhs, compiler)?;
                let rhs_val = self.evaluate_node(*rhs, compiler)?;
                self.evaluate_binary_op(lhs_val, op, rhs_val, compiler)
            }
            
            AstNode::Call { head, args } => {
                let call_target = compiler.call_resolution.get(head);
                if let Some(&crate::CallTarget::Function(fun_id)) = call_target {
                    let fun = &compiler.functions[fun_id.0];
                    if fun.is_comptime && fun.body.is_some() {
                        return self.evaluate_function_call(fun, args, compiler);
                    }
                }
                None
            }
            
            AstNode::If { condition, then_block, else_expression } => {
                let cond_val = self.evaluate_node(*condition, compiler)?;
                if let ComptimeValue::Bool(true) = cond_val {
                    self.evaluate_node(*then_block, compiler)
                } else if let Some(else_expr) = else_expression {
                    self.evaluate_node(*else_expr, compiler)
                } else {
                    Some(ComptimeValue::Unit)
                }
            }
            
            AstNode::Block(block_id) => {
                let mut result = Some(ComptimeValue::Unit);
                for node in &compiler.blocks[block_id.0].nodes {
                    if matches!(compiler.get_node(*node), AstNode::Return(..)) {
                        result = self.evaluate_node(*node, compiler);
                        break;
                    } else {
                        result = self.evaluate_node(*node, compiler);
                    }
                }
                result
            }
            
            AstNode::Return(return_expr) => {
                return_expr.as_ref().and_then(|e| self.evaluate_node(*e, compiler))
            }
            
            AstNode::Let { initializer, .. } => {
                if let Some(init) = initializer {
                    let val = self.evaluate_node(*init, compiler);
                    if let Some(v) = val {
                        self.values.insert(node_id, v.clone());
                    }
                    val
                } else {
                    None
                }
            }
            
            AstNode::UnaryOp { op, operand } => {
                let operand_val = self.evaluate_node(*operand, compiler)?;
                self.evaluate_unary_op(op, operand_val)
            }
            
            AstNode::Match { target, match_arms } => {
                let target_val = self.evaluate_node(*target, compiler)?;
                self.evaluate_match(target_val, match_arms, compiler)
            }
            
            _ => None,
        }
    }

    fn evaluate_binary_op(
        &self,
        lhs: ComptimeValue,
        op: &NodeId,
        rhs: ComptimeValue,
        _compiler: &crate::Compiler,
    ) -> Option<ComptimeValue> {
        match (lhs, op, rhs) {
            (ComptimeValue::Int(l), _, ComptimeValue::Int(r)) => {
                if matches!(self.compiler_get_node(op), AstNode::Add) {
                    Some(ComptimeValue::Int(l + r))
                } else if matches!(self.compiler_get_node(op), AstNode::Subtract) {
                    Some(ComptimeValue::Int(l - r))
                } else if matches!(self.compiler_get_node(op), AstNode::Multiply) {
                    Some(ComptimeValue::Int(l * r))
                } else if matches!(self.compiler_get_node(op), AstNode::Divide) {
                    if r != 0 { Some(ComptimeValue::Int(l / r)) } else { None }
                } else if matches!(self.compiler_get_node(op), AstNode::Modulo) {
                    if r != 0 { Some(ComptimeValue::Int(l % r)) } else { None }
                } else if matches!(self.compiler_get_node(op), AstNode::LessThan) {
                    Some(ComptimeValue::Bool(l < r))
                } else if matches!(self.compiler_get_node(op), AstNode::GreaterThan) {
                    Some(ComptimeValue::Bool(l > r))
                } else if matches!(self.compiler_get_node(op), AstNode::LessThanOrEqual) {
                    Some(ComptimeValue::Bool(l <= r))
                } else if matches!(self.compiler_get_node(op), AstNode::GreaterThanOrEqual) {
                    Some(ComptimeValue::Bool(l >= r))
                } else if matches!(self.compiler_get_node(op), AstNode::Equal) {
                    Some(ComptimeValue::Bool(l == r))
                } else if matches!(self.compiler_get_node(op), AstNode::NotEqual) {
                    Some(ComptimeValue::Bool(l != r))
                } else if matches!(self.compiler_get_node(op), AstNode::ShiftLeft) {
                    Some(ComptimeValue::Int(l << r))
                } else if matches!(self.compiler_get_node(op), AstNode::ShiftRight) {
                    Some(ComptimeValue::Int(l >> r))
                } else if matches!(self.compiler_get_node(op), AstNode::BitwiseAnd) {
                    Some(ComptimeValue::Int(l & r))
                } else if matches!(self.compiler_get_node(op), AstNode::BitwiseOr) {
                    Some(ComptimeValue::Int(l | r))
                } else if matches!(self.compiler_get_node(op), AstNode::BitwiseXor) {
                    Some(ComptimeValue::Int(l ^ r))
                } else {
                    None
                }
            }
            (ComptimeValue::Float(l), _, ComptimeValue::Float(r)) => {
                if matches!(self.compiler_get_node(op), AstNode::Add) {
                    Some(ComptimeValue::Float(l + r))
                } else if matches!(self.compiler_get_node(op), AstNode::Subtract) {
                    Some(ComptimeValue::Float(l - r))
                } else if matches!(self.compiler_get_node(op), AstNode::Multiply) {
                    Some(ComptimeValue::Float(l * r))
                } else if matches!(self.compiler_get_node(op), AstNode::Divide) {
                    if r != 0.0 { Some(ComptimeValue::Float(l / r)) } else { None }
                } else {
                    None
                }
            }
            (ComptimeValue::Bool(l), _, ComptimeValue::Bool(r)) => {
                if matches!(self.compiler_get_node(op), AstNode::And) {
                    Some(ComptimeValue::Bool(l && r))
                } else if matches!(self.compiler_get_node(op), AstNode::Or) {
                    Some(ComptimeValue::Bool(l || r))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn evaluate_unary_op(&self, op: &NodeId, operand: ComptimeValue) -> Option<ComptimeValue> {
        match (op, operand) {
            (_, ComptimeValue::Int(n)) => {
                if matches!(self.compiler_get_node(op), AstNode::Negate) {
                    Some(ComptimeValue::Int(-n))
                } else if matches!(self.compiler_get_node(op), AstNode::BitwiseNot) {
                    Some(ComptimeValue::Int(!n))
                } else {
                    None
                }
            }
            (_, ComptimeValue::Float(n)) => {
                if matches!(self.compiler_get_node(op), AstNode::Negate) {
                    Some(ComptimeValue::Float(-n))
                } else {
                    None
                }
            }
            (_, ComptimeValue::Bool(b)) => {
                if matches!(self.compiler_get_node(op), AstNode::Not) {
                    Some(ComptimeValue::Bool(!b))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn evaluate_function_call(
        &mut self,
        fun: &crate::Function,
        args: &[NodeId],
        compiler: &crate::Compiler,
    ) -> Option<ComptimeValue> {
        let arg_values: Vec<ComptimeValue> = args.iter()
            .filter_map(|&arg| self.evaluate_node(arg, compiler))
            .collect();

        if arg_values.len() != args.len() {
            return None;
        }

        if let Some(body) = fun.body {
            for (param, arg_val) in fun.params.iter().zip(arg_values.iter()) {
                self.values.insert(param.var_id.0.into(), arg_val.clone());
            }
            self.evaluate_node(body, compiler)
        } else {
            None
        }
    }

    fn evaluate_match(
        &self,
        _target: ComptimeValue,
        _match_arms: &[(NodeId, NodeId)],
        _compiler: &crate::Compiler,
    ) -> Option<ComptimeValue> {
        None
    }

    fn compiler_get_node(&self, _node_id: NodeId) -> &AstNode {
        &AstNode::None
    }
}

#[derive(Debug, Clone)]
pub struct ComptimeError {
    pub message: String,
    pub node_id: NodeId,
}

pub fn evaluate_comptime(compiler: &mut crate::Compiler) {
    let mut evaluator = ComptimeEvaluator::new();
    evaluator.evaluate(compiler);
}
