use crate::{ast, lexer::{self, Token}};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("Unexpected token: expected {expected}, found {found}")]
    UnexpectedToken {
        expected: String,
        found: String,
    },
    #[error("Unexpected end of input")]
    UnexpectedEof,
    #[error("Invalid syntax: {message}")]
    InvalidSyntax {
        message: String,
    },
    #[error("Unsupported feature: {feature}")]
    UnsupportedFeature {
        feature: String,
    },
    #[error("Lexical error: {0}")]
    LexicalError(#[from] lexer::LexerError),
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<ast::Program, ParserError> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        Ok(ast::Program { statements })
    }

    fn parse_statement(&mut self) -> Result<ast::Statement, ParserError> {
        let token = self.peek()?;
        match token {
            Token::Let | Token::Var | Token::LetZh | Token::VarZh => {
                self.parse_let_statement()
            },
            Token::Fn => {
                self.parse_function_statement()
            },
            Token::Struct | Token::StructZh => {
                self.parse_struct_statement()
            },
            Token::Enum | Token::EnumZh => {
                self.parse_enum_statement()
            },
            Token::Group | Token::GroupZh => {
                self.parse_group_statement()
            },
            Token::Return | Token::ReturnZh => {
                self.parse_return_statement()
            },
            Token::If | Token::IfZh | Token::Elif | Token::ElifZh => {
                // 消费If或Elif关键字
                self.advance()?;
                let expr = self.parse_if_expression()?;
                Ok(ast::Statement::Expression(expr))
            },
            Token::While | Token::WhileZh | Token::LoopZh => {
                self.parse_while_statement()
            },
            Token::For => {
                self.parse_for_statement()
            },
            Token::Import => {
                self.parse_import_statement()
            },
            Token::Match | Token::MatchZh => {
                // 消费Match关键字
                self.advance()?;
                let expr = self.parse_match_expression()?;
                Ok(ast::Statement::Expression(expr))
            },
            Token::Else | Token::ElseZh => {
                // else 关键字不能作为单独的语句，只能出现在 if 或 elif 之后
                return Err(ParserError::UnexpectedToken {
                    expected: "if or elif statement".to_string(),
                    found: format!("{:?}", token),
                });
            },
            _ => {
                // 表达式语句
                let expr = self.parse_expression()?;
                // 检查是否有分号，如果有则消费
                if self.check(Token::Semicolon) {
                    self.advance()?;
                }
                Ok(ast::Statement::Expression(expr))
            },
        }
    }

    fn parse_let_statement(&mut self) -> Result<ast::Statement, ParserError> {
        let token = self.advance()?;
        let mut mutable = matches!(token, Token::Var | Token::VarZh);
        
        // 检查是否有mut关键字
        if !mutable && self.check(Token::Mut) {
            self.advance()?;
            mutable = true;
        }
        
        let name = match self.advance()? {
            Token::Identifier(name) => name,
            token => return Err(ParserError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: format!("{:?}", token),
            }),
        };
        
        let ty = if self.check(Token::Colon) {
            self.advance()?;
            Some(self.parse_type()?)
        } else {
            None
        };
        
        self.expect(Token::Assign)?;
        let value = self.parse_expression()?;
        
        if self.check(Token::Semicolon) {
            self.advance()?;
        }
        
        Ok(ast::Statement::Let {
            mutable,
            name,
            ty,
            value,
        })
    }

    fn parse_function_statement(&mut self) -> Result<ast::Statement, ParserError> {
        // 检查是否有@kernel注解
        let mut is_kernel = false;
        if self.check(Token::KernelAttr) {
            self.advance()?; // 消费@kernel
            is_kernel = true;
        }
        
        self.advance()?; // 消费 fn
        
        let name = match self.advance()? {
            Token::Identifier(name) => name,
            token => return Err(ParserError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: format!("{:?}", token),
            }),
        };
        
        self.expect(Token::LParen)?;
        let params = self.parse_parameters()?;
        self.expect(Token::RParen)?;
        
        let return_type = if self.check(Token::Arrow) {
            self.advance()?;
            Some(self.parse_type()?)
        } else {
            None
        };
        
        let body = if self.check(Token::Assign) {
            // 表达式风格: fn name(params) -> type = expression;
            self.advance()?;
            self.parse_expression()?
        } else if self.check(Token::LBrace) {
            // 块风格: fn name(params) -> type { statements }
            self.advance()?; // 消费 {
            let mut statements = Vec::new();
            while !self.check(Token::RBrace) {
                statements.push(self.parse_statement()?);
            }
            self.advance()?; // 消费 }
            ast::Expression::Block(statements)
        } else if self.check(Token::Colon) {
            // 缩进风格: fn name(params) -> type:
            //     statement1
            //     statement2
            self.advance()?; // 消费 :
            // 解析单个语句或块
            let stmt = self.parse_statement()?;
            // 将单个语句转换为块表达式
            ast::Expression::Block(vec![stmt])
        } else {
            return Err(ParserError::UnexpectedToken {
                expected: "Assign, LBrace or Colon".to_string(),
                found: format!("{:?}", self.peek()?),
            });
        };
        
        if self.check(Token::Semicolon) {
            self.advance()?;
        }
        
        Ok(ast::Statement::Function {
            name,
            params,
            return_type,
            body,
            kernel: is_kernel,
        })
    }

    fn parse_struct_statement(&mut self) -> Result<ast::Statement, ParserError> {
        self.advance()?; // 消费 struct
        
        let name = match self.advance()? {
            Token::Identifier(name) => name,
            token => return Err(ParserError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: format!("{:?}", token),
            }),
        };
        
        self.expect(Token::LBrace)?;
        let mut fields = Vec::new();
        while !self.check(Token::RBrace) {
            let field_name = match self.advance()? {
                Token::Identifier(name) => name,
                token => return Err(ParserError::UnexpectedToken {
                    expected: "identifier".to_string(),
                    found: format!("{:?}", token),
                }),
            };
            
            self.expect(Token::Colon)?;
            let field_type = self.parse_type()?;
            
            fields.push(ast::StructField {
                name: field_name,
                ty: field_type,
            });
            
            // 检查是否有逗号，如果有则消费
            if self.check(Token::Comma) {
                self.advance()?;
            }
        }
        self.advance()?; // 消费 }
        
        if self.check(Token::Semicolon) {
            self.advance()?;
        }
        
        Ok(ast::Statement::Struct {
            name,
            fields,
        })
    }

    fn parse_enum_statement(&mut self) -> Result<ast::Statement, ParserError> {
        self.advance()?; // 消费 enum
        
        let name = match self.advance()? {
            Token::Identifier(name) => name,
            token => return Err(ParserError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: format!("{:?}", token),
            }),
        };
        
        self.expect(Token::LBrace)?;
        let mut variants = Vec::new();
        while !self.check(Token::RBrace) {
            let variant_name = match self.advance()? {
                Token::Identifier(name) => name,
                // 允许关键字作为变体名称
                Token::ErrorKeyword => "Error".to_string(),
                Token::Ok => "Ok".to_string(),
                Token::Some => "Some".to_string(),
                Token::None => "None".to_string(),
                Token::Box => "Box".to_string(),
                Token::Rc => "Rc".to_string(),
                Token::Arc => "Arc".to_string(),
                token => return Err(ParserError::UnexpectedToken {
                    expected: "identifier".to_string(),
                    found: format!("{:?}", token),
                }),
            };
            
            let fields = if self.check(Token::LParen) {
                self.advance()?;
                let mut field_list = Vec::new();
                while !self.check(Token::RParen) {
                    let field_name = match self.advance()? {
                        Token::Identifier(name) => name,
                        token => return Err(ParserError::UnexpectedToken {
                            expected: "identifier".to_string(),
                            found: format!("{:?}", token),
                        }),
                    };
                    
                    self.expect(Token::Colon)?;
                    let field_type = self.parse_type()?;
                    
                    field_list.push(ast::StructField {
                        name: field_name,
                        ty: field_type,
                    });
                    
                    if self.check(Token::Comma) {
                        self.advance()?;
                    } else if !self.check(Token::RParen) {
                        return Err(ParserError::UnexpectedToken {
                            expected: ", or )"
                                .to_string(),
                            found: format!("{:?}", self.peek()?),
                        });
                    }
                }
                self.advance()?; // 消费 )
                Some(field_list)
            } else {
                None
            };
            
            variants.push((variant_name, fields));
            
            // 检查是否有逗号，如果有则消费
            if self.check(Token::Comma) {
                self.advance()?;
            }
        }
        self.advance()?; // 消费 }
        
        if self.check(Token::Semicolon) {
            self.advance()?;
        }
        
        Ok(ast::Statement::Enum {
            name,
            variants,
        })
    }

    fn parse_group_statement(&mut self) -> Result<ast::Statement, ParserError> {
        self.advance()?; // 消费 group
        
        let name = match self.advance()? {
            Token::Identifier(name) => name,
            token => return Err(ParserError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: format!("{:?}", token),
            }),
        };
        
        self.expect(Token::LBrace)?;
        let mut members = Vec::new();
        while !self.check(Token::RBrace) {
            members.push(self.parse_statement()?);
        }
        self.advance()?; // 消费 }
        
        if self.check(Token::Semicolon) {
            self.advance()?;
        }
        
        Ok(ast::Statement::Group {
            name,
            members,
        })
    }

    fn parse_return_statement(&mut self) -> Result<ast::Statement, ParserError> {
        // 支持英文和中文关键字
        let token = self.advance()?;
        match token {
            Token::Return | Token::ReturnZh => (),
            _ => return Err(ParserError::UnexpectedToken {
                expected: "return or 返回".to_string(),
                found: format!("{:?}", token),
            }),
        }
        
        let value = if !self.check(Token::Semicolon) && !self.is_at_end() {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        if self.check(Token::Semicolon) {
            self.advance()?;
        }
        
        Ok(ast::Statement::Return(value))
    }

    fn parse_while_statement(&mut self) -> Result<ast::Statement, ParserError> {
        // 支持英文和中文关键字
        let token = self.advance()?;
        match token {
            Token::While | Token::WhileZh | Token::LoopZh => (),
            _ => return Err(ParserError::UnexpectedToken {
                expected: "while, 当, or 循环".to_string(),
                found: format!("{:?}", token),
            }),
        }
        
        let condition = self.parse_expression()?;
        self.expect(Token::Colon)?;
        let body = self.parse_expression()?;
        
        if self.check(Token::Semicolon) {
            self.advance()?;
        }
        
        Ok(ast::Statement::While {
            condition,
            body,
        })
    }

    fn parse_for_statement(&mut self) -> Result<ast::Statement, ParserError> {
        self.advance()?; // 消费 for
        
        let pattern = match self.advance()? {
            Token::Identifier(name) => name,
            token => return Err(ParserError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: format!("{:?}", token),
            }),
        };
        
        self.expect(Token::In)?;
        let in_expr = self.parse_expression()?;
        self.expect(Token::Colon)?;
        let body = self.parse_expression()?;
        
        if self.check(Token::Semicolon) {
            self.advance()?;
        }
        
        Ok(ast::Statement::For {
            pattern,
            in_expr,
            body,
        })
    }

    fn parse_import_statement(&mut self) -> Result<ast::Statement, ParserError> {
        self.advance()?; // 消费 import
        
        let path = match self.advance()? {
            Token::String(path) => path,
            token => return Err(ParserError::UnexpectedToken {
                expected: "string literal".to_string(),
                found: format!("{:?}", token),
            }),
        };
        
        let statement = if self.check(Token::As) {
            self.advance()?;
            let alias = match self.advance()? {
                Token::Identifier(alias) => alias,
                token => return Err(ParserError::UnexpectedToken {
                    expected: "identifier".to_string(),
                    found: format!("{:?}", token),
                }),
            };
            ast::Statement::ImportAs(path, alias)
        } else {
            ast::Statement::Import(path)
        };
        
        if self.check(Token::Semicolon) {
            self.advance()?;
        }
        
        Ok(statement)
    }

    fn parse_expression(&mut self) -> Result<ast::Expression, ParserError> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<ast::Expression, ParserError> {
        let left = self.parse_ternary()?;
        
        // 检查是否是赋值运算符
        if self.check(Token::Assign) || self.check(Token::AddAssign) || 
           self.check(Token::SubAssign) || self.check(Token::MulAssign) || 
           self.check(Token::DivAssign) || self.check(Token::ModAssign) {
            let op = self.advance()?;
            let right = self.parse_assignment()?;
            
            // 对于复合赋值，转换为二元运算
            let expr = match op {
                Token::Assign => ast::Expression::Assign {
                    left: Box::new(left),
                    right: Box::new(right),
                },
                Token::AddAssign => ast::Expression::Assign {
                    left: Box::new(left.clone()),
                    right: Box::new(ast::Expression::BinaryOp {
                        left: Box::new(left),
                        op: ast::BinaryOperator::Add,
                        right: Box::new(right),
                    }),
                },
                Token::SubAssign => ast::Expression::Assign {
                    left: Box::new(left.clone()),
                    right: Box::new(ast::Expression::BinaryOp {
                        left: Box::new(left),
                        op: ast::BinaryOperator::Sub,
                        right: Box::new(right),
                    }),
                },
                Token::MulAssign => ast::Expression::Assign {
                    left: Box::new(left.clone()),
                    right: Box::new(ast::Expression::BinaryOp {
                        left: Box::new(left),
                        op: ast::BinaryOperator::Mul,
                        right: Box::new(right),
                    }),
                },
                Token::DivAssign => ast::Expression::Assign {
                    left: Box::new(left.clone()),
                    right: Box::new(ast::Expression::BinaryOp {
                        left: Box::new(left),
                        op: ast::BinaryOperator::Div,
                        right: Box::new(right),
                    }),
                },
                Token::ModAssign => ast::Expression::Assign {
                    left: Box::new(left.clone()),
                    right: Box::new(ast::Expression::BinaryOp {
                        left: Box::new(left),
                        op: ast::BinaryOperator::Mod,
                        right: Box::new(right),
                    }),
                },
                _ => unreachable!(),
            };
            
            return Ok(expr);
        }
        
        Ok(left)
    }

    fn parse_ternary(&mut self) -> Result<ast::Expression, ParserError> {
        let expr = self.parse_logical_or()?;
        Ok(expr)
    }

    fn parse_logical_or(&mut self) -> Result<ast::Expression, ParserError> {
        let mut left = self.parse_logical_and()?;
        
        while self.check(Token::Or) {
            let _op = self.advance()?;
            let right = self.parse_logical_and()?;
            left = ast::Expression::BinaryOp {
                left: Box::new(left),
                op: ast::BinaryOperator::Or,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_logical_and(&mut self) -> Result<ast::Expression, ParserError> {
        let mut left = self.parse_equality()?;
        
        while self.check(Token::And) {
            let _op = self.advance()?;
            let right = self.parse_equality()?;
            left = ast::Expression::BinaryOp {
                left: Box::new(left),
                op: ast::BinaryOperator::And,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<ast::Expression, ParserError> {
        let mut left = self.parse_comparison()?;
        
        while self.check(Token::Eq) || self.check(Token::Ne) {
            let op = self.advance()?;
            let right = self.parse_comparison()?;
            left = ast::Expression::BinaryOp {
                left: Box::new(left),
                op: match op {
                    Token::Eq => ast::BinaryOperator::Eq,
                    Token::Ne => ast::BinaryOperator::Ne,
                    _ => unreachable!(),
                },
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<ast::Expression, ParserError> {
        let mut left = self.parse_range()?;
        
        while self.check(Token::Lt) || self.check(Token::Le) || 
              self.check(Token::Gt) || self.check(Token::Ge) {
            let op = self.advance()?;
            let right = self.parse_range()?;
            left = ast::Expression::BinaryOp {
                left: Box::new(left),
                op: match op {
                    Token::Lt => ast::BinaryOperator::Lt,
                    Token::Le => ast::BinaryOperator::Le,
                    Token::Gt => ast::BinaryOperator::Gt,
                    Token::Ge => ast::BinaryOperator::Ge,
                    _ => unreachable!(),
                },
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_range(&mut self) -> Result<ast::Expression, ParserError> {
        let left = self.parse_term()?;
        
        if self.check(Token::Range) || self.check(Token::RangeInclusive) {
            let op = self.advance()?;
            let right = self.parse_term()?;
            return Ok(ast::Expression::Range {
                start: Box::new(left),
                end: Box::new(right),
                inclusive: matches!(op, Token::RangeInclusive),
            });
        }
        
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<ast::Expression, ParserError> {
        let mut left = self.parse_factor()?;
        
        while self.check(Token::Plus) || self.check(Token::Minus) {
            let op = self.advance()?;
            let right = self.parse_factor()?;
            left = ast::Expression::BinaryOp {
                left: Box::new(left),
                op: match op {
                    Token::Plus => ast::BinaryOperator::Add,
                    Token::Minus => ast::BinaryOperator::Sub,
                    _ => unreachable!(),
                },
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<ast::Expression, ParserError> {
        let mut left = self.parse_cast()?;
        
        while self.check(Token::Star) || self.check(Token::Slash) || 
              self.check(Token::Percent) {
            let op = self.advance()?;
            let right = self.parse_cast()?;
            left = ast::Expression::BinaryOp {
                left: Box::new(left),
                op: match op {
                    Token::Star => ast::BinaryOperator::Mul,
                    Token::Slash => ast::BinaryOperator::Div,
                    Token::Percent => ast::BinaryOperator::Mod,
                    _ => unreachable!(),
                },
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    // 解析类型转换: expr as type
    fn parse_cast(&mut self) -> Result<ast::Expression, ParserError> {
        let mut expr = self.parse_unary()?;
        
        // 处理类型转换
        while self.check(Token::As) {
            self.advance()?; // 消费 as
            let ty = self.parse_type()?;
            // 将类型转换表示为一个特殊的函数调用
            expr = ast::Expression::Call {
                callee: Box::new(ast::Expression::Identifier("as".to_string())),
                args: vec![expr, ast::Expression::Identifier(ty)],
            };
        }
        
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<ast::Expression, ParserError> {
        if self.check(Token::Not) || self.check(Token::Minus) || 
           self.check(Token::Ampersand) || self.check(Token::Star) {
            let op = self.advance()?;
            let expr = self.parse_unary()?;
            return Ok(ast::Expression::UnaryOp {
                op: match op {
                    Token::Not => ast::UnaryOperator::Not,
                    Token::Minus => ast::UnaryOperator::Neg,
                    Token::Ampersand => ast::UnaryOperator::Ref,
                    Token::Star => ast::UnaryOperator::Deref,
                    _ => unreachable!(),
                },
                expr: Box::new(expr),
            });
        }
        
        self.parse_primary()
    }
    
    // 解析路径表达式，如 a::b::c 或 Status::Success(data)
    fn parse_path(&mut self) -> Result<ast::Expression, ParserError> {
        // 首先检查是否为标识符
        let token = self.peek()?;
        let path_token = match token {
            Token::Identifier(_) => self.advance()?,
            _ => return Err(ParserError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: format!("{:?}", token),
            }),
        };
        let mut path = match path_token {
            Token::Identifier(name) => name,
            _ => unreachable!(),
        };
        
        // 处理连续的 :: 操作符，如 a::b::c
        while self.check(Token::Colon) && self.check_nth(1, Token::Colon) {
            // 消费两个冒号
            self.advance()?;
            self.advance()?;
            
            // 解析下一个标识符
            let next_token = self.advance()?;
            let next_id = match next_token {
                Token::Identifier(name) => name,
                Token::ErrorKeyword => "Error".to_string(),
                Token::Ok => "Ok".to_string(),
                Token::Some => "Some".to_string(),
                Token::None => "None".to_string(),
                Token::Box => "Box".to_string(),
                Token::Rc => "Rc".to_string(),
                Token::Arc => "Arc".to_string(),
                token => return Err(ParserError::UnexpectedToken {
                    expected: "identifier".to_string(),
                    found: format!("{:?}", token),
                }),
            };
            
            // 构建完整路径
            path = format!("{}::{}", path, next_id);
        }
        
        // 将路径转换为标识符表达式
        let path_expr = ast::Expression::Identifier(path);
        
        // 现在处理可能的函数调用或字段访问
        let result = self.parse_call_expr_from_path(path_expr)?;
        Ok(result)
    }
    
    // 从路径表达式处理函数调用和后缀操作
    fn parse_call_expr_from_path(&mut self, expr: ast::Expression) -> Result<ast::Expression, ParserError> {
        self.parse_postfix(expr)
    }

    fn parse_postfix(&mut self, mut expr: ast::Expression) -> Result<ast::Expression, ParserError> {
        loop {
            if self.check(Token::LParen) {
                self.advance()?; // 消费 (
                let mut args = Vec::new();
                
                if !self.check(Token::RParen) {
                    loop {
                        args.push(self.parse_expression()?);
                        if !self.check(Token::Comma) {
                            break;
                        }
                        self.advance()?;
                    }
                }
                
                self.expect(Token::RParen)?;
                expr = ast::Expression::Call {
                    callee: Box::new(expr),
                    args,
                };
            } else if self.check(Token::LBracket) {
                self.advance()?; // 消费 [
                let index = self.parse_expression()?;
                self.expect(Token::RBracket)?;
                expr = ast::Expression::Index {
                    array: Box::new(expr),
                    index: Box::new(index),
                };
            } else if self.check(Token::Dot) {
                // 处理字段访问: expr.field
                self.advance()?; // 消费 .
                let field = match self.advance()? {
                    Token::Identifier(field) => field,
                    token => return Err(ParserError::UnexpectedToken {
                        expected: "identifier".to_string(),
                        found: format!("{:?}", token),
                    }),
                };
                expr = ast::Expression::FieldAccess {
                    expr: Box::new(expr),
                    field,
                };
            } else if self.check(Token::LBrace) {
                // 处理结构体实例化: Type { field: value }
                self.advance()?; // 消费 {
                
                // 解析字段列表
                let mut fields = Vec::new();
                while !self.check(Token::RBrace) {
                    // 解析字段名
                    let field_name = match self.advance()? {
                        Token::Identifier(name) => name,
                        token => return Err(ParserError::UnexpectedToken {
                            expected: "field name".to_string(),
                            found: format!("{:?}", token),
                        }),
                    };
                    
                    // 解析冒号
                    self.expect(Token::Colon)?;
                    
                    // 解析字段值
                    let field_value = self.parse_expression()?;
                    
                    // 添加到字段列表
                    fields.push((field_name, field_value));
                    
                    // 检查是否有逗号
                    if self.check(Token::Comma) {
                        self.advance()?;
                    } else if !self.check(Token::RBrace) {
                        break;
                    }
                }
                
                // 消费 }
                self.expect(Token::RBrace)?;
                
                // 构建结构体表达式
                match expr {
                    ast::Expression::Identifier(type_name) => {
                        expr = ast::Expression::Struct {
                            name: type_name,
                            fields,
                        };
                    },
                    _ => return Err(ParserError::UnexpectedToken {
                        expected: "struct type name".to_string(),
                        found: format!("{:?}", expr),
                    }),
                }
            } else {
                break;
            }
        }
        
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<ast::Expression, ParserError> {
        let token = self.peek()?;
        let expr = match token {
            // 字面量
            Token::Integer(_) | Token::Float(_) | Token::String(_) | 
            Token::True | Token::TrueZh | Token::False | Token::FalseZh | 
            Token::Null | Token::UnitLiteral(_) => {
                // 消费并处理字面量
                let actual_token = self.advance()?;
                match actual_token {
                    Token::Integer(n) => ast::Expression::Literal(ast::Literal::Integer(n)),
                    Token::Float(n) => ast::Expression::Literal(ast::Literal::Float(n)),
                    Token::String(s) => ast::Expression::Literal(ast::Literal::String(s)),
                    Token::True | Token::TrueZh => ast::Expression::Literal(ast::Literal::Boolean(true)),
                    Token::False | Token::FalseZh => ast::Expression::Literal(ast::Literal::Boolean(false)),
                    Token::Null => ast::Expression::Literal(ast::Literal::Null),
                    Token::UnitLiteral(unit_str) => {
                        // 解析字符串为元组 (value, unit)
                        let clean_str = unit_str.trim_matches(|c| c == '(' || c == ')');
                        let parts: Vec<&str> = clean_str.split(',').collect();
                        if parts.len() == 2 {
                            let value_str = parts[0].trim();
                            let unit_str = parts[1].trim();
                            if let Ok(value) = value_str.parse::<f64>() {
                                ast::Expression::Literal(ast::Literal::UnitLiteral(value, unit_str.to_string()))
                            } else {
                                return Err(ParserError::InvalidSyntax { message: format!("Invalid unit literal: {}", unit_str) });
                            }
                        } else {
                            return Err(ParserError::InvalidSyntax { message: format!("Invalid unit literal: {}", unit_str) });
                        }
                    },
                    _ => unreachable!(),
                }
            },
            // 输出函数
            Token::Print | Token::Println | Token::PrintZh | Token::PrintlnZh => {
                let actual_token = self.advance()?;
                ast::Expression::Identifier(match actual_token {
                    Token::Print => "print".to_string(),
                    Token::Println => "println".to_string(),
                    Token::PrintZh => "print".to_string(),
                    Token::PrintlnZh => "println".to_string(),
                    _ => unreachable!(),
                })
            },
            // 块表达式
            Token::LBrace => {
                self.advance()?; // 消费 {
                let mut statements = Vec::new();
                while !self.check(Token::RBrace) {
                    statements.push(self.parse_statement()?);
                }
                self.advance()?; // 消费 }
                ast::Expression::Block(statements)
            },
            // 数组表达式
            Token::LBracket => {
                self.advance()?; // 消费 [
                let mut elements = Vec::new();
                if !self.check(Token::RBracket) {
                    loop {
                        elements.push(self.parse_expression()?);
                        if !self.check(Token::Comma) {
                            break;
                        }
                        self.advance()?;
                    }
                }
                self.expect(Token::RBracket)?;
                ast::Expression::Array(elements)
            },

            // Return语句作为表达式
            Token::Return => {
                self.advance()?; // 消费 return
                // 返回语句作为表达式处理
                let value = if !self.check(Token::Semicolon) && !self.is_at_end() && 
                              !self.check(Token::RBrace) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                ast::Expression::Block(vec![ast::Statement::Return(value)])
            },
            // 标识符或路径表达式
            Token::Identifier(_) => {
                // 检查是否为路径表达式（如 a::b::c）
                let current_token = self.advance()?;
                let ident = match current_token {
                    Token::Identifier(name) => name,
                    _ => unreachable!(),
                };
                
                // 如果后续有 :: 操作符，则继续解析路径
                if self.check(Token::Colon) && self.check_nth(1, Token::Colon) {
                    // 回退一个位置，让parse_path处理
                    self.current -= 1;
                    self.parse_path()?
                } else {
                    ast::Expression::Identifier(ident)
                }
            },
            // Lambda表达式 (参数) => 表达式
            Token::LParen => {
                // 检查是否为lambda表达式
                // 保存当前位置
                let start_pos = self.current;
                self.advance()?; // 消费 (
                
                // 尝试解析参数
                if self.check(Token::RParen) {
                    // 空参数列表 ()
                    self.advance()?; // 消费 )
                    
                    if self.check(Token::FatArrow) {
                        // 是lambda表达式: () => body
                        self.advance()?; // 消费 =>
                        let body = self.parse_expression()?;
                        ast::Expression::Lambda {
                            params: Vec::new(),
                            body: Box::new(body),
                        }
                    } else {
                        // 不是lambda，是普通括号表达式
                        ast::Expression::Identifier("()".to_string())
                    }
                } else {
                    // 有内容，检查是否为标识符或类型
                    let next_token = self.peek()?;
                    match next_token {
                        Token::Identifier(_) | Token::Colon => {
                            // 可能是lambda参数列表
                            // 解析参数
                            self.current = start_pos; // 重置位置
                            self.advance()?; // 消费 (
                            
                            let mut params = Vec::new();
                            if !self.check(Token::RParen) {
                                loop {
                                    // 解析参数名
                                    let param_name = match self.advance()? {
                                        Token::Identifier(name) => name,
                                        _ => return Err(ParserError::UnexpectedToken {
                                            expected: "parameter name".to_string(),
                                            found: format!("{:?}", self.peek()?),
                                        }),
                                    };
                                    
                                    // 解析参数类型
                                    let param_type = if self.check(Token::Colon) {
                                        self.advance()?;
                                        Some(self.parse_type()?)
                                    } else {
                                        None
                                    };
                                    
                                    params.push(ast::Parameter {
                                        name: param_name,
                                        ty: param_type,
                                        default: None,
                                    });
                                    
                                    if !self.check(Token::Comma) {
                                        break;
                                    }
                                    self.advance()?; // 消费 ,
                                }
                            }
                            
                            self.expect(Token::RParen)?; // 消费 )
                            
                            if self.check(Token::FatArrow) {
                                // 是lambda表达式: (params) => body
                                self.advance()?; // 消费 =>
                                let body = self.parse_expression()?;
                                ast::Expression::Lambda {
                                    params,
                                    body: Box::new(body),
                                }
                            } else {
                                // 不是lambda，是普通括号表达式
                                ast::Expression::Identifier("(".to_string())
                            }
                        },
                        _ => {
                            // 不是lambda，是普通括号表达式
                            self.current = start_pos; // 重置位置
                            self.advance()?; // 消费 (
                            let expr = self.parse_expression()?;
                            self.expect(Token::RParen)?;
                            expr
                        },
                    }
                }
            },
            // 其他情况
            _ => {
                return Err(ParserError::UnexpectedToken {
                    expected: "expression".to_string(),
                    found: format!("{:?}", token),
                });
            },
        };
        
        // 处理后缀操作
        self.parse_postfix(expr)
    }

    fn parse_if_expression(&mut self) -> Result<ast::Expression, ParserError> {
        // 解析条件表达式，使用parse_assignment而不是parse_expression来避免无限递归
        let condition = self.parse_ternary()?;
        self.expect(Token::Colon)?;
        let then_branch = self.parse_expression()?;
        
        // 处理elif分支
        let mut else_branch = None;
        if self.check(Token::Elif) || self.check(Token::ElifZh) {
            // 将elif转换为嵌套的if-else
            self.advance()?;
            let elif_expr = self.parse_if_expression()?;
            else_branch = Some(Box::new(elif_expr));
        } else if self.check(Token::Else) || self.check(Token::ElseZh) {
            self.advance()?;
            self.expect(Token::Colon)?;
            else_branch = Some(Box::new(self.parse_expression()?));
        }
        
        Ok(ast::Expression::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    // 解析模式（用于match表达式的模式匹配）
    fn parse_pattern(&mut self) -> Result<ast::Expression, ParserError> {
        // 目前简单实现，只支持标识符和字面量
        let token = self.peek()?;
        let pattern = match token {
            // 字面量模式
            Token::Integer(_) | Token::Float(_) | Token::String(_) | 
            Token::True | Token::TrueZh | Token::False | Token::FalseZh | 
            Token::Null => {
                // 消费并处理字面量
                let actual_token = self.advance()?;
                match actual_token {
                    Token::Integer(n) => ast::Expression::Literal(ast::Literal::Integer(n)),
                    Token::Float(n) => ast::Expression::Literal(ast::Literal::Float(n)),
                    Token::String(s) => ast::Expression::Literal(ast::Literal::String(s)),
                    Token::True | Token::TrueZh => ast::Expression::Literal(ast::Literal::Boolean(true)),
                    Token::False | Token::FalseZh => ast::Expression::Literal(ast::Literal::Boolean(false)),
                    Token::Null => ast::Expression::Literal(ast::Literal::Null),
                    _ => unreachable!(),
                }
            },
            // 下划线模式
            Token::Identifier(_) if self.peek()? == Token::Identifier("_".to_string()) => {
                self.advance()?;
                ast::Expression::Identifier("_".to_string())
            },
            // 标识符模式或路径模式（如 Status::Success）
            Token::Identifier(_) => {
                // 直接调用parse_path处理
                self.parse_path()?
            },
            // 括号模式
            Token::LParen => {
                self.advance()?; // 消费 (
                let pattern = self.parse_pattern()?;
                self.expect(Token::RParen)?;
                pattern
            },
            // 其他情况
            _ => {
                return Err(ParserError::UnexpectedToken {
                    expected: "pattern".to_string(),
                    found: format!("{:?}", token),
                });
            },
        };
        
        Ok(pattern)
    }
    
    fn parse_match_expression(&mut self) -> Result<ast::Expression, ParserError> {
        let expr = self.parse_expression()?;
        self.expect(Token::Colon)?;
        
        let mut cases = Vec::new();
        while !self.is_at_end() && 
              !matches!(self.peek()?, Token::RParen | Token::RBrace | Token::Else) {
            let pattern = self.parse_pattern()?;
            let guard = if self.check(Token::If) {
                self.advance()?;
                Some(self.parse_expression()?)
            } else {
                None
            };
            
            self.expect(Token::FatArrow)?;
            let body = self.parse_expression()?;
            
            cases.push(ast::MatchCase {
                pattern,
                guard,
                body,
            });
            
            if self.check(Token::Comma) {
                self.advance()?;
            }
        }
        
        Ok(ast::Expression::Match {
            expr: Box::new(expr),
            cases,
        })
    }

    fn parse_parameters(&mut self) -> Result<Vec<ast::Parameter>, ParserError> {
        let mut params = Vec::new();
        
        if !self.check(Token::RParen) {
            loop {
                let name = match self.advance()? {
                    Token::Identifier(name) => name,
                    token => return Err(ParserError::UnexpectedToken {
                        expected: "identifier".to_string(),
                        found: format!("{:?}", token),
                    }),
                };
                
                let ty = if self.check(Token::Colon) {
                    self.advance()?;
                    Some(self.parse_type()?)
                } else {
                    None
                };
                
                let default = if self.check(Token::Assign) {
                    self.advance()?;
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                
                params.push(ast::Parameter {
                    name,
                    ty,
                    default,
                });
                
                if !self.check(Token::Comma) {
                    break;
                }
                self.advance()?;
            }
        }
        
        Ok(params)
    }

    fn parse_type(&mut self) -> Result<String, ParserError> {
        // 检查类型修饰符
        let mut modifier = None;
        if self.check(Token::Ref) || self.check(Token::RefZh) {
            self.advance()?;
            modifier = Some("ref".to_string());
            
            // 检查是否还有mut
            if self.check(Token::Mut) {
                self.advance()?;
                modifier = Some("mut ref".to_string());
            }
        }
        
        // 检查生命周期参数
        let mut lifetime = None;
        if self.check(Token::Lifetime) {
            self.advance()?;
            lifetime = Some("'a".to_string());
        }
        
        // 解析基础类型或引用类型
        let base_type = if self.check(Token::And) {
            // & 引用操作符
            self.advance()?;
            self.parse_type()?
        } else if self.check(Token::AndMut) {
            // &mut 可变引用操作符
            self.advance()?;
            format!("mut {}", self.parse_type()?)
        } else {
            // 处理内置类型关键字或标识符
            let ty_name = match self.advance()? {
                Token::List => "List".to_string(),
                Token::Result => "Result".to_string(),
                Token::Option => "Option".to_string(),
                Token::Identifier(ty) => ty,
                token => return Err(ParserError::UnexpectedToken {
                    expected: "type".to_string(),
                    found: format!("{:?}", token),
                }),
            };
            ty_name
        };
        
        // 处理泛型类型
        let generic_type = if self.check(Token::LBracket) {
            self.advance()?; // 消费[
            let mut inner_types = Vec::new();
            
            // 解析第一个类型参数
            inner_types.push(self.parse_type()?);
            
            // 解析剩余的类型参数
            while self.check(Token::Comma) {
                self.advance()?; // 消费,
                inner_types.push(self.parse_type()?);
            }
            
            self.expect(Token::RBracket)?; // 消费]
            
            format!("{}[{}]", base_type, inner_types.join(", "))
        } else {
            base_type
        };
        
        // 组装最终类型字符串
        let mut result = String::new();
        
        // 添加类型修饰符
        if let Some(type_modifier) = modifier {
            result.push_str(&type_modifier);
        }
        
        // 添加生命周期
        if let Some(lt) = lifetime {
            result.push_str(&lt);
        }
        
        // 添加类型
        result.push_str(&generic_type);
        
        Ok(result)
    }

    fn advance(&mut self) -> Result<Token, ParserError> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn peek(&mut self) -> Result<Token, ParserError> {
        if self.is_at_end() {
            Err(ParserError::UnexpectedEof)
        } else {
            Ok(self.tokens[self.current].clone())
        }
    }

    fn previous(&mut self) -> Result<Token, ParserError> {
        if self.current == 0 {
            Err(ParserError::UnexpectedEof)
        } else {
            Ok(self.tokens[self.current - 1].clone())
        }
    }

    fn check(&mut self, token_type: Token) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.tokens[self.current] == token_type
        }
    }
    
    // 检查下n个标记是否匹配
    fn check_nth(&mut self, n: usize, token_type: Token) -> bool {
        if self.current + n >= self.tokens.len() {
            false
        } else {
            self.tokens[self.current + n] == token_type
        }
    }

    fn expect(&mut self, token_type: Token) -> Result<(), ParserError> {
        if self.check(token_type.clone()) {
            self.advance()?;
            Ok(())
        } else {
            Err(ParserError::UnexpectedToken {
                expected: format!("{:?}", token_type),
                found: format!("{:?}", self.peek()?),
            })
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }
}