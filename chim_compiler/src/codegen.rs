use crate::ast;
use crate::ir::{self, Module, Function, Instruction, Type as IRType};

pub trait CodeGenerator {
    fn generate_module(&mut self, program: &ast::Program) -> ir::Module;
    fn generate_function(&mut self, func: &ast::Statement) -> Option<Function>;
    fn generate_expression(&mut self, expr: &ast::Expression) -> Vec<Instruction>;
}

pub struct IRGenerator {
    pub module: ir::Module,
    pub current_function: Option<Function>,
    pub temp_counter: usize,
    pub label_counter: usize,
    pub last_temp: Option<String>,  // 追踪最后生成的临时变量
    pub param_names: Vec<String>,    // 记录当前函数的参数名
}

impl IRGenerator {
    pub fn new() -> Self {
        Self {
            module: ir::Module::new(),
            current_function: None,
            temp_counter: 0,
            label_counter: 0,
            last_temp: None,
            param_names: Vec::new(),
        }
    }
    
    pub fn fresh_temp(&mut self) -> String {
        self.temp_counter += 1;
        let temp = format!(".tmp{}", self.temp_counter);
        self.last_temp = Some(temp.clone());
        temp
    }
    
    pub fn fresh_label(&mut self) -> String {
        self.label_counter += 1;
        format!(".L{}", self.label_counter)
    }
    
    pub fn convert_type(type_str: &str) -> IRType {
        match type_str {
            "int" | "i32" => IRType::Int32,
            "int64" => IRType::Int64,
            "float" | "f32" => IRType::Float32,
            "float64" | "f64" => IRType::Float64,
            "bool" => IRType::Bool,
            "string" => IRType::String,
            "void" => IRType::Void,
            t if t.starts_with("&") => {
                let inner = &t[1..];
                IRType::Ref(Box::new(IRGenerator::convert_type(inner.trim())))
            },
            t if t.starts_with("&mut ") => {
                let inner = &t[5..];
                IRType::MutRef(Box::new(IRGenerator::convert_type(inner.trim())))
            },
            t if t.starts_with("List[") => {
                let inner = &t[5..t.len()-1];
                IRType::Array(Box::new(IRGenerator::convert_type(inner)), 0)
            },
            t if t.starts_with("ref ") => {
                let inner = &t[4..];
                IRType::Ref(Box::new(IRGenerator::convert_type(inner.trim())))
            },
            _ => IRType::Int32,
        }
    }
    
    fn generate_ecs_entity(&mut self, name: &str, components: &[String]) -> Option<Function> {
        let mut func = Function::new(
            format!("{}_create", name),
            IRType::Int32,
        );
        func.is_public = true;
        
        let entity_var = self.fresh_temp();
        func.body.push(Instruction::Call {
            dest: Some(entity_var.clone()),
            func: "__ecs_entity_create".to_string(),
            args: vec![],
        });
        
        for comp in components {
            let _init_call = Instruction::Call {
                dest: None,
                func: format!("__ecs_component_init_{}", comp),
                args: vec![entity_var.clone()],
            };
            func.body.push(_init_call);
        }
        
        func.body.push(Instruction::Return(Some(entity_var)));
        
        Some(func)
    }
    
    fn generate_ecs_component(&mut self, name: &str, fields: &[ast::StructField]) -> Option<Function> {
        let mut struct_ = ir::Struct::new(name.to_string());
        for field in fields {
            let ty = IRGenerator::convert_type(&field.ty);
            struct_.fields.push((field.name.clone(), ty));
        }
        self.module.add_struct(struct_);
        
        let mut init_func = Function::new(
            format!("__ecs_component_init_{}", name),
            IRType::Void,
        );
        init_func.params.push(("entity_id".to_string(), IRType::Int32));
        
        let comp_ptr = self.fresh_temp();
        init_func.body.push(Instruction::Alloca {
            dest: comp_ptr.clone(),
            ty: IRType::Ptr(Box::new(IRType::Void)),
        });
        
        for field in fields {
            let field_ptr = self.fresh_temp();
            init_func.body.push(Instruction::GetElementPtr {
                dest: field_ptr.clone(),
                src: comp_ptr.clone(),
                indices: vec![0],
            });
        }
        
        init_func.body.push(Instruction::Return(None));
        
        Some(init_func)
    }
    
    fn generate_ecs_system(&mut self, name: &str, query: &[String], body: &ast::Expression) -> Option<Function> {
        let mut func = Function::new(
            name.to_string(),
            IRType::Void,
        );
        func.is_public = true;
        
        func.params.push(("query_ctx".to_string(), IRType::Ptr(Box::new(IRType::Void))));
        
        let loop_label = self.fresh_label();
        let body_label = self.fresh_label();
        let exit_label = self.fresh_label();
        
        let query_iter = self.fresh_temp();
        func.body.push(Instruction::Call {
            dest: Some(query_iter.clone()),
            func: "__ecs_query_create".to_string(),
            args: query.iter().map(|c| format!("\"{}\"", c)).collect(),
        });
        
        func.body.push(Instruction::Label(loop_label.clone()));
        
        let has_next = self.fresh_temp();
        func.body.push(Instruction::Call {
            dest: Some(has_next.clone()),
            func: "__ecs_query_has_next".to_string(),
            args: vec![query_iter.clone()],
        });
        
        func.body.push(Instruction::CondBr {
            cond: has_next,
            true_bb: body_label.clone(),
            false_bb: exit_label.clone(),
        });
        
        func.body.push(Instruction::Label(body_label.clone()));
        
        let current_entity = self.fresh_temp();
        func.body.push(Instruction::Call {
            dest: Some(current_entity.clone()),
            func: "__ecs_query_get_entity".to_string(),
            args: vec![query_iter.clone()],
        });
        
        let body_insts = self.generate_expression(body);
        func.body.extend(body_insts);
        
        func.body.push(Instruction::Br(loop_label));
        
        func.body.push(Instruction::Label(exit_label.clone()));
        
        func.body.push(Instruction::Call {
            dest: None,
            func: "__ecs_query_destroy".to_string(),
            args: vec![query_iter.clone()],
        });
        
        func.body.push(Instruction::Return(None));
        
        Some(func)
    }
}

impl CodeGenerator for IRGenerator {
    fn generate_module(&mut self, program: &ast::Program) -> ir::Module {
        for stmt in &program.statements {
            if let Some(func) = self.generate_function(stmt) {
                self.module.add_function(func);
            }
        }
        self.module.clone()
    }
    
    fn generate_function(&mut self, stmt: &ast::Statement) -> Option<Function> {
        match stmt {
            ast::Statement::Function { name, params, return_type, body, .. } => {
                let mut func = Function::new(
                    name.clone(),
                    IRGenerator::convert_type(return_type.as_deref().unwrap_or("void"))
                );
                func.is_public = true;
                
                // 保存参数名
                self.param_names.clear();
                
                // 转换参数
                for param in params {
                    let ty = IRGenerator::convert_type(
                        param.ty.as_deref().unwrap_or("int")
                    );
                    func.params.push((param.name.clone(), ty));
                    self.param_names.push(param.name.clone());
                }
                
                // 保存当前函数并生成函数体
                let old_func = self.current_function.take();
                self.current_function = Some(func.clone());
                
                let body_insts = self.generate_expression(body);
                func.body = body_insts;
                
                // 恢复之前函数
                self.current_function = old_func;
                self.param_names.clear();
                
                Some(func)
            },
            ast::Statement::Struct { name, fields } => {
                let mut struct_ = ir::Struct::new(name.clone());
                for field in fields {
                    let ty = IRGenerator::convert_type(&field.ty);
                    struct_.fields.push((field.name.clone(), ty));
                }
                self.module.add_struct(struct_);
                None
            },
            // ECS声明
            ast::Statement::Entity { name, components } => {
                self.generate_ecs_entity(name, components)
            },
            ast::Statement::Component { name, fields } => {
                self.generate_ecs_component(name, fields)
            },
            ast::Statement::System { name, query, body } => {
                self.generate_ecs_system(name, query, body)
            },
            _ => None,
        }
    }
    
    fn generate_expression(&mut self, expr: &ast::Expression) -> Vec<Instruction> {
        let mut insts = Vec::new();
        
        match expr {
            ast::Expression::Literal(lit) => {
                let dest = self.fresh_temp();
                match lit {
                    ast::Literal::Integer(n) => {
                        // 对于整数字面量，直接使用数值，不生成Load指令
                        self.last_temp = Some(n.to_string());
                    },
                    ast::Literal::Float(f) => {
                        self.last_temp = Some(f.to_string());
                    },
                    ast::Literal::String(s) => {
                        insts.push(Instruction::Load {
                            dest: dest.clone(),
                            src: format!("\"{}\" /* string */", s),
                        });
                    },
                    ast::Literal::Boolean(b) => {
                        self.last_temp = Some(if *b { "1" } else { "0" }.to_string());
                    },
                    _ => {}
                }
                insts
            },
            
            ast::Expression::Identifier(name) => {
                // 如果是函数参数或已定义的变量，直接使用名字，不生成Load
                if self.param_names.contains(name) {
                    self.last_temp = Some(name.clone());
                } else {
                    let dest = self.fresh_temp();
                    insts.push(Instruction::Load {
                        dest: dest.clone(),
                        src: name.clone(),
                    });
                }
                insts
            },
            
            ast::Expression::BinaryOp { left, op, right } => {
                insts.extend(self.generate_expression(left));
                let left_temp = self.last_temp.clone().unwrap_or_else(|| "0".to_string());
                
                insts.extend(self.generate_expression(right));
                let right_temp = self.last_temp.clone().unwrap_or_else(|| "0".to_string());
                
                let dest = self.fresh_temp();
                
                let op_inst = match op {
                    ast::BinaryOperator::Add => Instruction::Add {
                        dest: dest.clone(),
                        left: left_temp,
                        right: right_temp,
                    },
                    ast::BinaryOperator::Sub => Instruction::Sub {
                        dest: dest.clone(),
                        left: left_temp,
                        right: right_temp,
                    },
                    ast::BinaryOperator::Mul => Instruction::Mul {
                        dest: dest.clone(),
                        left: left_temp,
                        right: right_temp,
                    },
                    ast::BinaryOperator::Div => Instruction::Div {
                        dest: dest.clone(),
                        left: left_temp,
                        right: right_temp,
                    },
                    ast::BinaryOperator::Mod => Instruction::Mod {
                        dest: dest.clone(),
                        left: left_temp,
                        right: right_temp,
                    },
                    ast::BinaryOperator::Eq => Instruction::Eq {
                        dest: dest.clone(),
                        left: left_temp,
                        right: right_temp,
                    },
                    ast::BinaryOperator::Ne => Instruction::Ne {
                        dest: dest.clone(),
                        left: left_temp,
                        right: right_temp,
                    },
                    ast::BinaryOperator::Lt => Instruction::Lt {
                        dest: dest.clone(),
                        left: left_temp,
                        right: right_temp,
                    },
                    ast::BinaryOperator::Le => Instruction::Le {
                        dest: dest.clone(),
                        left: left_temp,
                        right: right_temp,
                    },
                    ast::BinaryOperator::Gt => Instruction::Gt {
                        dest: dest.clone(),
                        left: left_temp,
                        right: right_temp,
                    },
                    ast::BinaryOperator::Ge => Instruction::Ge {
                        dest: dest.clone(),
                        left: left_temp,
                        right: right_temp,
                    },
                    ast::BinaryOperator::And => Instruction::And {
                        dest: dest.clone(),
                        left: left_temp,
                        right: right_temp,
                    },
                    ast::BinaryOperator::Or => Instruction::Or {
                        dest: dest.clone(),
                        left: left_temp,
                        right: right_temp,
                    },
                    _ => Instruction::Nop,
                };
                
                insts.push(op_inst);
                insts
            },
            
            ast::Expression::Call { callee, args } => {
                let mut arg_temps = Vec::new();
                for arg in args {
                    insts.extend(self.generate_expression(arg));
                    if let Some(ref temp) = self.last_temp {
                        arg_temps.push(temp.clone());
                    }
                }
                
                let dest = if let ast::Expression::Identifier(name) = callee.as_ref() {
                    if name != "print" && name != "println" {
                        Some(self.fresh_temp())
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                insts.push(Instruction::Call {
                    dest: dest.clone(),
                    func: if let ast::Expression::Identifier(name) = callee.as_ref() {
                        name.clone()
                    } else {
                        "unknown".to_string()
                    },
                    args: arg_temps,
                });
                
                insts
            },
            
            ast::Expression::If { condition, then_branch, else_branch } => {
                insts.extend(self.generate_expression(condition));
                let cond_temp = self.last_temp.clone().unwrap_or_else(|| "0".to_string());
                
                let then_label = self.fresh_label();
                let else_label = self.fresh_label();
                let end_label = self.fresh_label();
                
                insts.push(Instruction::CondBr {
                    cond: cond_temp,
                    true_bb: then_label.clone(),
                    false_bb: else_label.clone(),
                });
                
                // Then分支
                insts.push(Instruction::Label(then_label));
                insts.extend(self.generate_expression(then_branch));
                insts.push(Instruction::Br(end_label.clone()));
                
                // Else分支
                insts.push(Instruction::Label(else_label));
                if let Some(else_b) = else_branch {
                    insts.extend(self.generate_expression(else_b));
                }
                insts.push(Instruction::Br(end_label.clone()));
                
                // End
                insts.push(Instruction::Label(end_label));
                
                insts
            },
            
            ast::Expression::Block(stmts) => {
                // 处理块中的每个语句
                for stmt in stmts {
                    match stmt {
                        ast::Statement::Expression(expr) => {
                            insts.extend(self.generate_expression(expr));
                        },
                        ast::Statement::Let { name, ty, value, .. } => {
                            insts.extend(self.generate_expression(value));
                            let src_temp = self.last_temp.clone().unwrap_or_else(|| "0".to_string());
                            
                            // 生成Alloca指令
                            let ir_type = if let Some(t) = ty {
                                IRGenerator::convert_type(t)
                            } else {
                                IRType::Int32  // 默认类型
                            };
                            insts.push(Instruction::Alloca {
                                dest: name.clone(),
                                ty: ir_type,
                            });
                            
                            // 生成Store指令
                            insts.push(Instruction::Store {
                                dest: name.clone(),
                                src: src_temp,
                            });
                        },
                        ast::Statement::Return(Some(expr)) => {
                            insts.extend(self.generate_expression(expr));
                            let ret_temp = self.last_temp.clone().unwrap_or_else(|| "0".to_string());
                            insts.push(Instruction::Return(Some(ret_temp)));
                        },
                        ast::Statement::Return(None) => {
                            insts.push(Instruction::Return(None));
                        },
                        _ => {}
                    }
                }
                insts
            },
            
            ast::Expression::UnaryOp { op, expr } => {
                insts.extend(self.generate_expression(expr));
                let src = self.last_temp.clone().unwrap_or_else(|| "0".to_string());
                
                let dest = self.fresh_temp();
                
                match op {
                    ast::UnaryOperator::Neg => {
                        insts.push(Instruction::Sub {
                            dest: dest.clone(),
                            left: "0".to_string(),
                            right: src,
                        });
                    },
                    ast::UnaryOperator::Not => {
                        insts.push(Instruction::Not {
                            dest: dest.clone(),
                            src,
                        });
                    },
                    ast::UnaryOperator::Ref => {
                        insts.push(Instruction::Borrow {
                            dest: dest.clone(),
                            src,
                            mutable: false,
                        });
                    },
                    ast::UnaryOperator::Deref => {
                        insts.push(Instruction::Deref {
                            dest: dest.clone(),
                            src,
                        });
                    },
                }
                
                insts
            },
            
            ast::Expression::Match { expr, cases } => {
                insts.extend(self.generate_expression(expr));
                let expr_temp = self.last_temp.clone().unwrap_or_else(|| "0".to_string());
                
                let end_label = self.fresh_label();
                let mut first_case = true;
                
                for case in cases {
                    let case_label = self.fresh_label();
                    
                    insts.push(Instruction::Label(case_label.clone()));
                    insts.extend(self.generate_expression(&case.body));
                    insts.push(Instruction::Br(end_label.clone()));
                    
                    if first_case {
                        first_case = false;
                    }
                }
                
                if !cases.is_empty() {
                    insts.push(Instruction::Label(end_label));
                }
                
                insts
            },
            
            ast::Expression::Array(exprs) => {
                for expr in exprs {
                    insts.extend(self.generate_expression(expr));
                }
                insts
            },
            
            ast::Expression::Index { array, index } => {
                insts.extend(self.generate_expression(array));
                let arr_temp = self.last_temp.clone().unwrap_or_else(|| "arr".to_string());
                
                insts.extend(self.generate_expression(index));
                let idx_temp = self.last_temp.clone().unwrap_or_else(|| "0".to_string());
                
                let dest = self.fresh_temp();
                
                insts.push(Instruction::GetElementPtr {
                    dest: dest.clone(),
                    src: arr_temp,
                    indices: vec![0, 0],
                });
                
                insts
            },
            
            ast::Expression::FieldAccess { expr, field: _ } => {
                insts.extend(self.generate_expression(expr));
                insts
            },
            
            _ => insts,
        }
    }
}

impl Default for IRGenerator {
    fn default() -> Self {
        Self::new()
    }
}
