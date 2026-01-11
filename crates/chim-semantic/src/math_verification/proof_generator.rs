use chim_ast::*;
use chim_span::Span;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Proof {
    pub statement: String,
    pub dependencies: Vec<Proof>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ProofGenerator {
    pub proofs: HashMap<Ident, Vec<Proof>>,
    pub current_proof: Vec<Proof>,
    pub errors: Vec<ProofGenerationError>,
}

#[derive(Debug, Clone)]
pub struct ProofGenerationError {
    pub kind: ProofGenerationErrorKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ProofGenerationErrorKind {
    CannotProveStatement(String),
    ProofCycleDetected,
    InvalidProofStructure,
    MissingProofDependency(Ident),
}

impl ProofGenerator {
    pub fn new() -> Self {
        ProofGenerator {
            proofs: HashMap::new(),
            current_proof: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn generate(&mut self, expr: &Expr) -> Result<Proof, Vec<ProofGenerationError>> {
        self.current_proof.clear();
        self.generate_proof(expr)?;
        if self.errors.is_empty() && !self.current_proof.is_empty() {
            Ok(self.current_proof.last().unwrap().clone())
        } else {
            Err(self.errors.clone())
        }
    }

    fn generate_proof(&mut self, expr: &Expr) -> Result<(), Vec<ProofGenerationError>> {
        match &expr.kind {
            ExprKind::Var(var_expr) => {
                self.generate_var_proof(&var_expr.name)?;
            }
            ExprKind::Binary(bin_expr) => {
                self.generate_binary_proof(bin_expr)?;
            }
            ExprKind::Unary(unary_expr) => {
                self.generate_unary_proof(unary_expr)?;
            }
            ExprKind::Call(call_expr) => {
                self.generate_call_proof(call_expr)?;
            }
            ExprKind::Block(block_expr) => {
                self.generate_block_proof(block_expr)?;
            }
            ExprKind::If(if_expr) => {
                self.generate_if_proof(if_expr)?;
            }
            ExprKind::Loop(loop_expr) => {
                self.generate_loop_proof(loop_expr)?;
            }
            ExprKind::Match(match_expr) => {
                self.generate_match_proof(match_expr)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn generate_var_proof(&mut self, var: &Ident) -> Result<(), Vec<ProofGenerationError>> {
        let proof = Proof {
            statement: format!("Variable {} exists", var),
            dependencies: Vec::new(),
            span: Span::default(),
        };
        self.current_proof.push(proof);
        Ok(())
    }

    fn generate_binary_proof(&mut self, bin_expr: &BinaryExpr) -> Result<(), Vec<ProofGenerationError>> {
        let left_proof = self.generate_subproof(&bin_expr.left)?;
        let right_proof = self.generate_subproof(&bin_expr.right)?;
        
        let proof = Proof {
            statement: format!("Binary operation: {:?}", bin_expr.op),
            dependencies: vec![left_proof, right_proof],
            span: Span::default(),
        };
        self.current_proof.push(proof);
        Ok(())
    }

    fn generate_unary_proof(&mut self, unary_expr: &UnaryExpr) -> Result<(), Vec<ProofGenerationError>> {
        let operand_proof = self.generate_subproof(&unary_expr.operand)?;
        
        let proof = Proof {
            statement: format!("Unary operation: {:?}", unary_expr.op),
            dependencies: vec![operand_proof],
            span: Span::default(),
        };
        self.current_proof.push(proof);
        Ok(())
    }

    fn generate_call_proof(&mut self, call_expr: &CallExpr) -> Result<(), Vec<ProofGenerationError>> {
        let callee_proof = self.generate_subproof(&call_expr.callee)?;
        let mut arg_proofs = Vec::new();
        for arg in &call_expr.args {
            arg_proofs.push(self.generate_subproof(arg)?);
        }
        
        let proof = Proof {
            statement: format!("Function call: {}", call_expr.callee),
            dependencies: vec![callee_proof].into_iter().chain(arg_proofs).collect(),
            span: Span::default(),
        };
        self.current_proof.push(proof);
        Ok(())
    }

    fn generate_block_proof(&mut self, block_expr: &BlockExpr) -> Result<(), Vec<ProofGenerationError>> {
        let mut stmt_proofs = Vec::new();
        for stmt in &block_expr.stmts {
            stmt_proofs.push(self.generate_stmt_proof(stmt)?);
        }
        if let Some(expr) = &block_expr.expr {
            stmt_proofs.push(self.generate_subproof(expr)?);
        }
        
        let proof = Proof {
            statement: "Block execution".to_string(),
            dependencies: stmt_proofs,
            span: Span::default(),
        };
        self.current_proof.push(proof);
        Ok(())
    }

    fn generate_if_proof(&mut self, if_expr: &IfExpr) -> Result<(), Vec<ProofGenerationError>> {
        let cond_proof = self.generate_subproof(&if_expr.condition)?;
        let then_proof = self.generate_subproof(&if_expr.then_branch)?;
        let else_proof = if let Some(else_branch) = &if_expr.else_branch {
            Some(self.generate_subproof(else_branch)?)
        } else {
            None
        };
        
        let proof = Proof {
            statement: "Conditional execution".to_string(),
            dependencies: vec![cond_proof, then_proof]
                .into_iter()
                .chain(else_proof)
                .filter_map(|p| p)
                .collect(),
            span: Span::default(),
        };
        self.current_proof.push(proof);
        Ok(())
    }

    fn generate_loop_proof(&mut self, loop_expr: &LoopExpr) -> Result<(), Vec<ProofGenerationError>> {
        let body_proof = self.generate_subproof(&loop_expr.body)?;
        
        let proof = Proof {
            statement: "Loop execution".to_string(),
            dependencies: vec![body_proof],
            span: Span::default(),
        };
        self.current_proof.push(proof);
        Ok(())
    }

    fn generate_match_proof(&mut self, match_expr: &MatchExpr) -> Result<(), Vec<ProofGenerationError>> {
        let value_proof = self.generate_subproof(&match_expr.value)?;
        let mut case_proofs = Vec::new();
        for case in &match_expr.cases {
            case_proofs.push(self.generate_subproof(&case.body)?);
        }
        
        let proof = Proof {
            statement: "Pattern matching".to_string(),
            dependencies: vec![value_proof].into_iter().chain(case_proofs).collect(),
            span: Span::default(),
        };
        self.current_proof.push(proof);
        Ok(())
    }

    fn generate_stmt_proof(&mut self, stmt: &Stmt) -> Result<Proof, Vec<ProofGenerationError>> {
        match stmt {
            Stmt::Let(let_stmt) => {
                if let Some(expr) = &let_stmt.value {
                    let value_proof = self.generate_subproof(expr)?;
                    let proof = Proof {
                        statement: format!("Variable declaration: {}", let_stmt.name),
                        dependencies: vec![value_proof],
                        span: Span::default(),
                    };
                    self.current_proof.push(proof);
                    Ok(proof)
                } else {
                    let proof = Proof {
                        statement: format!("Variable declaration: {}", let_stmt.name),
                        dependencies: Vec::new(),
                        span: Span::default(),
                    };
                    self.current_proof.push(proof);
                    Ok(proof)
                }
            }
            Stmt::Expr(expr_stmt) => {
                self.generate_subproof(&expr_stmt.expr)?;
                let proof = Proof {
                    statement: "Expression statement".to_string(),
                    dependencies: Vec::new(),
                    span: Span::default(),
                };
                self.current_proof.push(proof);
                Ok(proof)
            }
            Stmt::Assign(assign_stmt) => {
                let value_proof = self.generate_subproof(&assign_stmt.value)?;
                let proof = Proof {
                    statement: format!("Assignment: {}", assign_stmt.target),
                    dependencies: vec![value_proof],
                    span: Span::default(),
                };
                self.current_proof.push(proof);
                Ok(proof)
            }
            _ => {
                let proof = Proof {
                    statement: "Statement execution".to_string(),
                    dependencies: Vec::new(),
                    span: Span::default(),
                };
                self.current_proof.push(proof);
                Ok(proof)
            }
        }
    }

    fn generate_subproof(&mut self, expr: &Expr) -> Result<Proof, Vec<ProofGenerationError>> {
        let start_len = self.current_proof.len();
        self.generate_proof(expr)?;
        if self.current_proof.len() > start_len {
            Ok(self.current_proof[start_len].clone())
        } else {
            Err(vec![ProofGenerationError {
                kind: ProofGenerationErrorKind::CannotProveStatement("No proof generated".to_string()),
                span: Span::default(),
            }])
        }
    }

    pub fn register_proof(&mut self, var: Ident, proof: Proof) {
        self.proofs.entry(var).or_insert_with(Vec::new).push(proof);
    }

    pub fn get_proof(&self, var: &Ident) -> Option<&Vec<Proof>> {
        self.proofs.get(var)
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    pub fn generate_item_proofs(&mut self, item: &Item) -> Result<(), Vec<ProofGenerationError>> {
        match item {
            Item::Function(func) => {
                for stmt in &func.body {
                    self.generate_stmt_proof(stmt)?;
                }
            }
            Item::Struct(_) => {}
            Item::Enum(_) => {}
            _ => {}
        }
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }
}

impl Default for ProofGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_generator_creation() {
        let generator = ProofGenerator::new();
        assert!(!generator.has_errors());
    }

    #[test]
    fn test_proof_registration() {
        let mut generator = ProofGenerator::new();
        let var = Arc::from("x");
        let proof = Proof {
            statement: "Test proof".to_string(),
            dependencies: Vec::new(),
            span: Span::default(),
        };
        generator.register_proof(var.clone(), proof);
        let proofs = generator.get_proof(&var);
        assert!(proofs.is_some());
        assert_eq!(proofs.unwrap().len(), 1);
    }

    #[test]
    fn test_error_clearing() {
        let mut generator = ProofGenerator::new();
        generator.errors.push(ProofGenerationError {
            kind: ProofGenerationErrorKind::CannotProveStatement("test".to_string()),
            span: Span::default(),
        });
        assert!(generator.has_errors());
        generator.clear_errors();
        assert!(!generator.has_errors());
    }
}
