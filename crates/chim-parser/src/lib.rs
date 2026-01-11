use chim_lexer::{Token, SpannedToken, TokenStream, Identifier};
use chim_ast::*;
use chim_span::{FileId, Span};
use chim_error::{ChimError, ErrorKind};
use std::sync::Arc;
use smallvec::SmallVec;

#[derive(Debug)]
pub struct Parser<'a> {
    tokens: TokenStream,
    interner: &'a mut lasso::Rodeo,
    file_id: FileId,
    errors: Vec<ChimError>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<SpannedToken>, interner: &'a mut lasso::Rodeo, file_id: FileId) -> Self {
        Parser {
            tokens: TokenStream::new(tokens, std::mem::take(interner)),
            interner,
            file_id,
            errors: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Program, Vec<ChimError>> {
        let mut items = Vec::new();

        while !self.tokens.at_end() {
            match self.parse_item() {
                Ok(Some(item)) => items.push(item),
                Ok(None) => {}
                Err(e) => {
                    self.errors.push(e);
                    self.recover();
                }
            }
        }

        if self.errors.is_empty() {
            Ok(Program {
                items,
                span: Span::new(self.file_id, 0, 0, 0, 0),
            })
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    fn parse_item(&mut self) -> Result<Option<Item>, ChimError> {
        let start_span = self.current_span()?;

        match self.tokens.peek().map(|t| &t.token) {
            Some(&Token::Func) => self.parse_function().map(Some),
            Some(&Token::Struct) => self.parse_struct().map(Some),
            Some(&Token::Enum) => self.parse_enum().map(Some),
            Some(&Token::Trait) => self.parse_trait().map(Some),
            Some(&Token::Impl) => self.parse_impl().map(Some),
            Some(&Token::Use) => self.parse_use().map(Some),
            Some(&Token::Mod) => self.parse_mod().map(Some),
            Some(&Token::Extern) => self.parse_extern().map(Some),
            Some(&Token::Const) => self.parse_constant().map(Some),
            Some(&Token::Static) => self.parse_static().map(Some),
            Some(&Token::Macro) => self.parse_macro().map(Some),
            Some(&Token::ForAll) => self.parse_forall().map(Some),
            Some(&Token::Default) => self.parse_default().map(Some),
            Some(&Token::Sync) => self.parse_sync().map(Some),
            Some(&Token::Sized) => self.parse_sized().map(Some),
            Some(&Token::IntoIterator) => self.parse_intoiterator().map(Some),
            Some(&Token::Let) | Some(&Token::LetAlt) | Some(&Token::Var) => {
                self.errors.push(ChimError::new(
                    ErrorKind::Parser,
                    "items cannot start with variable declarations".to_string(),
                ).with_span(start_span));
                let err = self.errors.last().cloned().unwrap_or_else(|| {
                    ChimError::new(ErrorKind::Parser, "unknown error".to_string())
                });
                Err(err)
            }
            _ => Ok(None),
        }
    }

    fn parse_function(&mut self) -> Result<Item, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let is_pub = self.parse_visibility()?;
        let is_async = self.tokens.peek().map(|t| &t.token) == Some(&Token::Async);
        if is_async {
            self.tokens.next();
        }

        let name = match self.parse_identifier() {
            Ok(n) => n,
            Err(e) => {
                self.report_error_with_context(
                    ErrorKind::Parser,
                    "failed to parse function name".to_string(),
                    start_span,
                    "function declaration"
                );
                return Err(e);
            }
        };
        
        let generics = self.parse_generics()?;
        
        let params = match self.parse_function_params() {
            Ok(p) => p,
            Err(e) => {
                self.errors.push(e);
                self.skip_to(Token::LBrace);
                Vec::new()
            }
        };
        
        let return_type = match self.parse_return_type() {
            Ok(t) => t,
            Err(e) => {
                self.errors.push(e);
                None
            }
        };
        
        let where_clauses = self.parse_where_clauses()?;
        
        let body = match self.parse_block() {
            Ok(b) => b,
            Err(e) => {
                self.errors.push(e);
                self.skip_to(Token::RBrace);
                Vec::new()
            }
        };

        let span = start_span.merge(&self.current_span().unwrap_or(start_span));

        Ok(Item::Function(Function {
            name: Arc::from(name),
            params,
            return_type,
            body,
            span,
            is_pub,
            is_async,
            lifetimes: generics,
            where_clauses,
        }))
    }

    fn parse_struct(&mut self) -> Result<Item, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let is_pub = self.parse_visibility()?;
        let name = self.parse_identifier()?;
        let generics = self.parse_generics()?;
        let where_clauses = self.parse_where_clauses()?;
        let fields = self.parse_struct_fields()?;

        let span = start_span.merge(&self.current_span().unwrap_or(start_span));

        Ok(Item::Struct(Struct {
            name: Arc::from(name),
            fields,
            span,
            is_pub,
            generics,
            where_clauses,
        }))
    }

    fn parse_enum(&mut self) -> Result<Item, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let is_pub = self.parse_visibility()?;
        let name = self.parse_identifier()?;
        let generics = self.parse_generics()?;
        let where_clauses = self.parse_where_clauses()?;
        let variants = self.parse_enum_variants()?;

        let span = start_span.merge(&self.current_span().unwrap_or(start_span));

        Ok(Item::Enum(Enum {
            name: Arc::from(name),
            variants,
            span,
            is_pub,
            generics,
            where_clauses,
        }))
    }

    fn parse_trait(&mut self) -> Result<Item, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let is_pub = self.parse_visibility()?;
        let name = self.parse_identifier()?;
        let generics = self.parse_generics()?;
        let super_traits = self.parse_trait_bounds()?;
        let where_clauses = self.parse_where_clauses()?;
        let items = self.parse_trait_items()?;

        let span = start_span.merge(&self.current_span().unwrap_or(start_span));

        Ok(Item::Trait(Trait {
            name: Arc::from(name),
            items,
            span,
            is_pub,
            generics,
            super_traits,
            where_clauses,
        }))
    }

    fn parse_impl(&mut self) -> Result<Item, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let generics = self.parse_generics()?;
        let trait_name = self.parse_optional_type_path()?;
        let type_name = self.parse_type()?;
        let where_clauses = self.parse_where_clauses()?;
        let items = self.parse_impl_items()?;

        let span = start_span.merge(&self.current_span().unwrap_or(start_span));

        Ok(Item::Impl(Impl {
            trait_name,
            type_name,
            items,
            span,
            generics,
            where_clauses,
        }))
    }

    fn parse_use(&mut self) -> Result<Item, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let is_pub = self.parse_visibility()?;
        let path = self.parse_path()?;
        let alias = if self.tokens.peek().map(|t| &t.token) == Some(&Token::As) {
            self.tokens.next();
            Some(Arc::from(self.parse_identifier()?))
        } else {
            None
        };
        self.expect(Token::Semicolon)?;

        let span = start_span.merge(&self.current_span().unwrap_or(start_span));

        Ok(Item::Use(Use {
            path,
            alias,
            span,
            is_pub,
        }))
    }

    fn parse_mod(&mut self) -> Result<Item, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let is_pub = self.parse_visibility()?;
        let name = self.parse_identifier()?;
        let body = if self.tokens.peek().map(|t| &t.token) == Some(&Token::LBrace) {
            self.tokens.next();
            let mut items = Vec::new();
            while self.tokens.peek().map(|t| &t.token) != Some(&Token::RBrace) {
                match self.parse_item() {
                    Ok(Some(item)) => items.push(item),
                    Ok(None) => {}
                    Err(e) => self.errors.push(e),
                }
            }
            self.expect(Token::RBrace)?;
            items
        } else {
            self.expect(Token::Semicolon)?;
            Vec::new()
        };

        let span = start_span.merge(&self.current_span().unwrap_or(start_span));

        Ok(Item::Mod(Mod {
            name: Arc::from(name),
            items: body,
            span,
            is_pub,
            file_path: None,
        }))
    }

    fn parse_extern(&mut self) -> Result<Item, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let abi = if let Some(&Token::String(_)) = self.tokens.peek().map(|t| &t.token) {
            let abi = self.intern_string();
            self.tokens.next();
            abi
        } else {
            "C".to_string()
        };

        self.expect(Token::LBrace)?;
        let mut items = Vec::new();
        while self.tokens.peek().map(|t| &t.token) != Some(&Token::RBrace) {
            let is_pub = self.parse_visibility()?;
            let name = self.parse_identifier()?;
            self.expect(Token::Colon)?;
            let ty = self.parse_type()?;
            self.expect(Token::Semicolon)?;

            items.push(ExternItem {
                name: Arc::from(name),
                ty,
                span: Span::new(self.file_id, 0, 0, 0, 0),
                is_pub,
            });
        }
        self.expect(Token::RBrace)?;

        let span = start_span.merge(&self.current_span().unwrap_or(start_span));

        Ok(Item::Extern(ExternBlock {
            abi,
            items,
            span,
        }))
    }

    fn parse_constant(&mut self) -> Result<Item, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let is_pub = self.parse_visibility()?;
        let name = self.parse_identifier()?;
        self.expect(Token::Colon)?;
        let ty = self.parse_type()?;
        self.expect(Token::Eq)?;
        let value = self.parse_expr()?;
        self.expect(Token::Semicolon)?;

        let span = start_span.merge(&self.current_span().unwrap_or(start_span));

        Ok(Item::Constant(Constant {
            name: Arc::from(name),
            ty: Some(ty),
            value,
            span,
            is_pub,
        }))
    }

    fn parse_static(&mut self) -> Result<Item, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let is_pub = self.parse_visibility()?;
        let is_mut = self.tokens.peek().map(|t| &t.token) == Some(&Token::Mut);
        if is_mut {
            self.tokens.next();
        }
        let name = self.parse_identifier()?;
        self.expect(Token::Colon)?;
        let ty = self.parse_type()?;
        let value = if self.tokens.peek().map(|t| &t.token) == Some(&Token::Eq) {
            self.tokens.next();
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect(Token::Semicolon)?;

        let span = start_span.merge(&self.current_span().unwrap_or(start_span));

        Ok(Item::Static(Static {
            name: Arc::from(name),
            ty,
            value,
            span,
            is_pub,
            is_mut,
        }))
    }

    fn parse_macro(&mut self) -> Result<Item, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let is_pub = self.parse_visibility()?;
        let is_procedural = self.tokens.peek().map(|t| &t.token) == Some(&Token::Procedural);
        if is_procedural {
            self.tokens.next();
        }

        let name = self.parse_identifier()?;
        let params = self.parse_macro_params()?;
        let body = self.parse_macro_body()?;
        self.expect(Token::Semicolon)?;

        let span = start_span.merge(&self.current_span().unwrap_or(start_span));

        Ok(Item::Macro(Macro {
            name: Arc::from(name),
            params,
            body,
            span,
            is_pub,
            is_procedural,
        }))
    }

    fn parse_macro_params(&mut self) -> Result<Vec<MacroParam>, ChimError> {
        self.expect(Token::LParen)?;
        let mut params = Vec::new();

        while self.tokens.peek().map(|t| &t.token) != Some(&Token::RParen) {
            let name = self.parse_identifier()?;
            let ty = if self.tokens.peek().map(|t| &t.token) == Some(&Token::Colon) {
                self.tokens.next();
                Some(self.parse_type()?)
            } else {
                None
            };
            params.push(MacroParam {
                name: Arc::from(name),
                ty,
                span: self.current_span()?,
            });
            if self.tokens.peek().map(|t| &t.token) == Some(&Token::Comma) {
                self.tokens.next();
            }
        }
        self.expect(Token::RParen)?;
        Ok(params)
    }

    fn parse_macro_body(&mut self) -> Result<MacroBody, ChimError> {
        if self.tokens.peek().map(|t| &t.token) == Some(&Token::LBrace) {
            self.tokens.next();
            let mut rules = Vec::new();
            while self.tokens.peek().map(|t| &t.token) != Some(&Token::RBrace) {
                let pattern = self.parse_macro_pattern()?;
                self.expect(Token::Arrow)?;
                let expansion = self.parse_macro_expansion()?;
                rules.push(MacroRule { pattern, expansion });
                if self.tokens.peek().map(|t| &t.token) == Some(&Token::Comma) {
                    self.tokens.next();
                }
            }
            self.expect(Token::RBrace)?;
            Ok(MacroBody::Rules(rules))
        } else {
            Ok(MacroBody::Procedural(String::new()))
        }
    }

    fn parse_macro_pattern(&mut self) -> Result<MacroPattern, ChimError> {
        Ok(MacroPattern::Token(MacroToken::Ident(Arc::from("test"))))
    }

    fn parse_macro_expansion(&mut self) -> Result<MacroExpansion, ChimError> {
        let expr = self.parse_expr()?;
        Ok(MacroExpansion::Expr(expr))
    }

    fn parse_forall(&mut self) -> Result<Item, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let is_pub = self.parse_visibility()?;
        let name = self.parse_identifier()?;
        let generics = self.parse_generics()?;
        let params = self.parse_function_params()?;
        let return_type = self.parse_return_type()?;
        let where_clauses = self.parse_where_clauses()?;
        let body = self.parse_block()?;

        let span = start_span.merge(&self.current_span().unwrap_or(start_span));

        Ok(Item::ForAll(ForAll {
            name: Arc::from(name),
            params,
            return_type,
            body,
            span,
            is_pub,
            generics,
            where_clauses,
        }))
    }

    fn parse_default(&mut self) -> Result<Item, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let is_pub = self.parse_visibility()?;
        let name = self.parse_identifier()?;
        self.expect(Token::Colon)?;
        let ty = self.parse_type()?;
        self.expect(Token::Eq)?;
        let value = self.parse_expr()?;
        self.expect(Token::Semicolon)?;

        let span = start_span.merge(&self.current_span().unwrap_or(start_span));

        Ok(Item::Default(Default {
            name: Arc::from(name),
            ty,
            value,
            span,
            is_pub,
        }))
    }

    fn parse_sync(&mut self) -> Result<Item, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let is_pub = self.parse_visibility()?;
        let name = self.parse_identifier()?;
        self.expect(Token::Colon)?;
        let ty = self.parse_type()?;
        self.expect(Token::Semicolon)?;

        let span = start_span.merge(&self.current_span().unwrap_or(start_span));

        Ok(Item::Sync(Sync {
            name: Arc::from(name),
            ty,
            span,
            is_pub,
        }))
    }

    fn parse_sized(&mut self) -> Result<Item, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let is_pub = self.parse_visibility()?;
        let name = self.parse_identifier()?;
        self.expect(Token::Colon)?;
        let ty = self.parse_type()?;
        self.expect(Token::Semicolon)?;

        let span = start_span.merge(&self.current_span().unwrap_or(start_span));

        Ok(Item::Sized(Sized {
            name: Arc::from(name),
            ty,
            span,
            is_pub,
        }))
    }

    fn parse_intoiterator(&mut self) -> Result<Item, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let is_pub = self.parse_visibility()?;
        let name = self.parse_identifier()?;
        self.expect(Token::Colon)?;
        let ty = self.parse_type()?;
        self.expect(Token::Semicolon)?;

        let span = start_span.merge(&self.current_span().unwrap_or(start_span));

        Ok(Item::IntoIterator(IntoIterator {
            name: Arc::from(name),
            ty,
            span,
            is_pub,
        }))
    }

    fn parse_function_params(&mut self) -> Result<Vec<Param>, ChimError> {
        self.expect(Token::LParen)?;
        let mut params = Vec::new();

        while self.tokens.peek().map(|t| &t.token) != Some(&Token::RParen) {
            let start_span = self.current_span()?;
            let is_mut = self.tokens.peek().map(|t| &t.token) == Some(&Token::Mut);
            if is_mut {
                self.tokens.next();
            }
            let is_ref = self.tokens.peek().map(|t| &t.token) == Some(&Token::Ref);
            if is_ref {
                self.tokens.next();
            }

            let name = self.parse_identifier()?;

            if self.tokens.peek().map(|t| &t.token) == Some(&Token::Colon) {
                self.tokens.next();
                let ty = self.parse_type()?;
                let span = start_span.merge(&self.current_span().unwrap_or(start_span));
                params.push(Param {
                    name: Arc::from(name),
                    ty,
                    span,
                    is_mut,
                    is_ref,
                });
            } else {
                let span = start_span.merge(&self.current_span().unwrap_or(start_span));
                params.push(Param {
                    name: Arc::from(name),
                    ty: Type {
                        kind: Box::new(TypeKind::Infer),
                        span,
                    },
                    span,
                    is_mut,
                    is_ref,
                });
            }

            if self.tokens.peek().map(|t| &t.token) == Some(&Token::Comma) {
                self.tokens.next();
            }
        }

        self.expect(Token::RParen)?;
        Ok(params)
    }

    fn parse_function_params_full(&mut self) -> Result<Vec<Param>, ChimError> {
        self.parse_function_params()
    }

    fn parse_return_type(&mut self) -> Result<Option<Type>, ChimError> {
        if self.tokens.peek().map(|t| &t.token) == Some(&Token::ThinArrow) {
            self.tokens.next();
            let ty = self.parse_type()?;
            Ok(Some(ty))
        } else {
            Ok(None)
        }
    }

    fn parse_struct_fields(&mut self) -> Result<Vec<Field>, ChimError> {
        self.expect(Token::LBrace)?;
        let mut fields = Vec::new();

        while self.tokens.peek().map(|t| &t.token) != Some(&Token::RBrace) {
            let start_span = self.current_span()?;
            let is_pub = self.parse_visibility()?;
            let name = self.parse_identifier()?;
            self.expect(Token::Colon)?;
            let ty = self.parse_type()?;
            self.expect(Token::Semicolon)?;

            let span = start_span.merge(&self.current_span().unwrap_or(start_span));
            fields.push(Field {
                name: Arc::from(name),
                ty,
                span,
                is_pub,
                attributes: Vec::new(),
            });
        }

        self.expect(Token::RBrace)?;
        Ok(fields)
    }

    fn parse_enum_variants(&mut self) -> Result<Vec<Variant>, ChimError> {
        self.expect(Token::LBrace)?;
        let mut variants = Vec::new();
        let mut index = 0i128;

        while self.tokens.peek().map(|t| &t.token) != Some(&Token::RBrace) {
            let start_span = self.current_span()?;
            let name = self.parse_identifier()?;

            let fields = if self.tokens.peek().map(|t| &t.token) == Some(&Token::LParen) {
                self.tokens.next();
                let mut fields = Vec::new();
                while self.tokens.peek().map(|t| &t.token) != Some(&Token::RParen) {
                    let field_name = self.parse_identifier()?;
                    self.expect(Token::Colon)?;
                    let field_ty = self.parse_type()?;

                    fields.push(Field {
                        name: Arc::from(field_name),
                        ty: field_ty,
                        span: Span::new(self.file_id, 0, 0, 0, 0),
                        is_pub: true,
                        attributes: Vec::new(),
                    });

                    if self.tokens.peek().map(|t| &t.token) == Some(&Token::Comma) {
                        self.tokens.next();
                    }
                }
                self.expect(Token::RParen)?;
                fields
            } else {
                Vec::new()
            };

            let span = start_span.merge(&self.current_span().unwrap_or(start_span));
            variants.push(Variant {
                name: Arc::from(name),
                fields,
                span,
                attributes: Vec::new(),
            });

            if self.tokens.peek().map(|t| &t.token) == Some(&Token::Comma) {
                self.tokens.next();
                index += 1;
            }
        }

        self.expect(Token::RBrace)?;
        Ok(variants)
    }

    fn parse_trait_items(&mut self) -> Result<Vec<TraitItem>, ChimError> {
        self.expect(Token::LBrace)?;
        let mut items = Vec::new();

        while self.tokens.peek().map(|t| &t.token) != Some(&Token::RBrace) {
            match self.tokens.peek().map(|t| &t.token) {
                Some(&Token::Func) => {
                    let func = self.parse_function_sig()?;
                    items.push(TraitItem::Function(func));
                }
                Some(&Token::Const) => {
                    self.tokens.next();
                    let name = self.parse_identifier()?;
                    self.expect(Token::Colon)?;
                    let ty = self.parse_type()?;
                    let default = if self.tokens.peek().map(|t| &t.token) == Some(&Token::Eq) {
                        self.tokens.next();
                        Some(self.parse_expr()?)
                    } else {
                        None
                    };
                    self.expect(Token::Semicolon)?;
                    items.push(TraitItem::Const(TraitConst {
                        name: Arc::from(name),
                        ty,
                        default,
                        span: Span::new(self.file_id, 0, 0, 0, 0),
                    }));
                }
                Some(&Token::Type) => {
                    self.tokens.next();
                    let name = self.parse_identifier()?;
                    let _bounds = self.parse_trait_bounds()?;
                    let default = if self.tokens.peek().map(|t| &t.token) == Some(&Token::Eq) {
                        self.tokens.next();
                        Some(self.parse_type()?)
                    } else {
                        None
                    };
                    self.expect(Token::Semicolon)?;
                    items.push(TraitItem::Type(TypeBinding {
                        name: Arc::from(name),
                        ty: default.unwrap_or(Type {
                            kind: Box::new(TypeKind::Infer),
                            span: Span::new(self.file_id, 0, 0, 0, 0),
                        }),
                        span: Span::new(self.file_id, 0, 0, 0, 0),
                    }));
                }
                _ => {
                    self.errors.push(ChimError::new(
                        ErrorKind::Parser,
                        "invalid trait item".to_string(),
                    ).with_span(self.current_span()?));
                    break;
                }
            }
        }

        self.expect(Token::RBrace)?;
        Ok(items)
    }

    fn parse_impl_items(&mut self) -> Result<Vec<ImplItem>, ChimError> {
        self.expect(Token::LBrace)?;
        let mut items = Vec::new();

        while self.tokens.peek().map(|t| &t.token) != Some(&Token::RBrace) {
            match self.tokens.peek().map(|t| &t.token) {
                Some(&Token::Func) => {
                    let func = self.parse_impl_function()?;
                    items.push(ImplItem::Function(func));
                }
                Some(&Token::Const) => {
                    self.tokens.next();
                    let _is_pub = self.parse_visibility()?;
                    let name = self.parse_identifier()?;
                    self.expect(Token::Colon)?;
                    let ty = self.parse_type()?;
                    self.expect(Token::Eq)?;
                    let value = self.parse_expr()?;
                    self.expect(Token::Semicolon)?;
                    items.push(ImplItem::Const(Constant {
                        name: Arc::from(name),
                        ty: Some(ty),
                        value,
                        span: Span::new(self.file_id, 0, 0, 0, 0),
                        is_pub: false,
                    }));
                }
                Some(&Token::Type) => {
                    self.tokens.next();
                    let name = self.parse_identifier()?;
                    self.expect(Token::Eq)?;
                    let ty = self.parse_type()?;
                    self.expect(Token::Semicolon)?;
                    items.push(ImplItem::Type(TypeBinding {
                        name: Arc::from(name),
                        ty,
                        span: Span::new(self.file_id, 0, 0, 0, 0),
                    }));
                }
                _ => {
                    self.errors.push(ChimError::new(
                        ErrorKind::Parser,
                        "invalid impl item".to_string(),
                    ).with_span(self.current_span()?));
                    break;
                }
            }
        }

        self.expect(Token::RBrace)?;
        Ok(items)
    }

    fn parse_function_sig(&mut self) -> Result<FunctionSig, ChimError> {
        let _ = self.tokens.next();

        let name = self.parse_identifier()?;
        let params = self.parse_function_params()?;
        let return_type = self.parse_return_type()?;

        Ok(FunctionSig {
            name: Arc::from(name),
            params,
            return_type,
            span: Span::new(self.file_id, 0, 0, 0, 0),
        })
    }

    fn parse_impl_function(&mut self) -> Result<Function, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let is_pub = self.parse_visibility()?;
        let name = self.parse_identifier()?;
        let params = self.parse_function_params()?;
        let return_type = self.parse_return_type()?;
        let body = self.parse_block()?;

        let span = start_span.merge(&self.current_span().unwrap_or(start_span));

        Ok(Function {
            name: Arc::from(name),
            params,
            return_type,
            body,
            span,
            is_pub,
            is_async: false,
            lifetimes: Vec::new(),
            where_clauses: Vec::new(),
        })
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ChimError> {
        let start_span = self.current_span()?;
        self.expect(Token::LBrace)?;
        let mut stmts = Vec::new();

        while self.tokens.peek().map(|t| &t.token) != Some(&Token::RBrace) {
            match self.parse_stmt() {
                Ok(stmt) => stmts.push(stmt),
                Err(e) => {
                    self.errors.push(e);
                    self.recover_in_block();
                }
            }
        }

        self.expect(Token::RBrace)?;
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ChimError> {
        let start_span = self.current_span()?;

        match self.tokens.peek().map(|t| &t.token) {
            Some(&Token::Let) | Some(&Token::LetAlt) => self.parse_let_stmt(),
            Some(&Token::Var) => self.parse_var_stmt(),
            Some(&Token::Return) => self.parse_return_stmt(),
            Some(&Token::Break) => self.parse_break_stmt(),
            Some(&Token::Continue) => self.parse_continue_stmt(),
            Some(&Token::Loop) => self.parse_loop_stmt(),
            Some(&Token::While) => self.parse_while_stmt(),
            Some(&Token::For) => self.parse_for_stmt(),
            Some(&Token::Match) => self.parse_match_stmt(),
            Some(&Token::LBrace) => {
                let body = self.parse_block()?;
                Ok(Stmt {
                    kind: StmtKind::Expr(Expr {
                        kind: ExprKind::Block(BlockExpr {
                            label: None,
                            stmts: body,
                            ty: None,
                        }),
                        span: start_span,
                        ty: None,
                    }),
                    span: start_span,
                })
            }
            _ => {
                let expr = self.parse_expr()?;
                if self.tokens.peek().map(|t| &t.token) == Some(&Token::Semicolon) {
                    self.tokens.next();
                }
                Ok(Stmt {
                    kind: StmtKind::Expr(expr),
                    span: start_span,
                })
            }
        }
    }

    fn parse_let_stmt(&mut self) -> Result<Stmt, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let pattern = self.parse_pattern()?;
        let ty = if self.tokens.peek().map(|t| &t.token) == Some(&Token::Colon) {
            self.tokens.next();
            Some(self.parse_type()?)
        } else {
            None
        };
        let initializer = if self.tokens.peek().map(|t| &t.token) == Some(&Token::Eq) {
            self.tokens.next();
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect(Token::Semicolon)?;

        Ok(Stmt {
            kind: StmtKind::Let(LetStmt {
                pattern,
                ty,
                initializer,
                span: start_span,
            }),
            span: start_span,
        })
    }

    fn parse_var_stmt(&mut self) -> Result<Stmt, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let pattern = self.parse_pattern()?;
        let ty = if self.tokens.peek().map(|t| &t.token) == Some(&Token::Colon) {
            self.tokens.next();
            Some(self.parse_type()?)
        } else {
            None
        };
        let initializer = if self.tokens.peek().map(|t| &t.token) == Some(&Token::Eq) {
            self.tokens.next();
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect(Token::Semicolon)?;

        Ok(Stmt {
            kind: StmtKind::Var(VarStmt {
                pattern,
                ty,
                initializer,
                span: start_span,
            }),
            span: start_span,
        })
    }

    fn parse_return_stmt(&mut self) -> Result<Stmt, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let value = if self.tokens.peek().map(|t| &t.token) == Some(&Token::Semicolon) {
            None
        } else {
            Some(Box::new(self.parse_expr()?))
        };
        self.expect(Token::Semicolon)?;

        Ok(Stmt {
            kind: StmtKind::Return(value),
            span: start_span,
        })
    }

    fn parse_break_stmt(&mut self) -> Result<Stmt, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let value = if self.tokens.peek().map(|t| &t.token) == Some(&Token::Semicolon) {
            None
        } else {
            Some(Box::new(self.parse_expr()?))
        };
        self.expect(Token::Semicolon)?;

        Ok(Stmt {
            kind: StmtKind::Break(None, value),
            span: start_span,
        })
    }

    fn parse_continue_stmt(&mut self) -> Result<Stmt, ChimError> {
        let _ = self.tokens.next();
        self.expect(Token::Semicolon)?;

        Ok(Stmt {
            kind: StmtKind::Continue,
            span: self.current_span()?,
        })
    }

    fn parse_loop_stmt(&mut self) -> Result<Stmt, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let body = self.parse_block()?;

        Ok(Stmt {
            kind: StmtKind::Loop(LoopStmt {
                label: None,
                body,
                span: start_span,
            }),
            span: start_span,
        })
    }

    fn parse_while_stmt(&mut self) -> Result<Stmt, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let condition = self.parse_expr()?;
        let body = self.parse_block()?;

        Ok(Stmt {
            kind: StmtKind::While(WhileStmt {
                label: None,
                condition,
                body,
                span: start_span,
            }),
            span: start_span,
        })
    }

    fn parse_for_stmt(&mut self) -> Result<Stmt, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let pattern = self.parse_pattern()?;
        self.expect(Token::In)?;
        let iterable = self.parse_expr()?;
        let body = self.parse_block()?;

        Ok(Stmt {
            kind: StmtKind::For(ForStmt {
                label: None,
                pattern,
                iterable,
                body,
                span: start_span,
            }),
            span: start_span,
        })
    }

    fn parse_match_stmt(&mut self) -> Result<Stmt, ChimError> {
        let _ = self.tokens.next();

        let start_span = self.current_span()?;
        let expr = match self.parse_expr() {
            Ok(e) => e,
            Err(e) => {
                self.report_error_with_context(
                    ErrorKind::Parser,
                    "failed to parse match expression".to_string(),
                    start_span,
                    "match statement"
                );
                self.skip_to(Token::LBrace);
                return Err(e);
            }
        };
        
        if let Err(e) = self.expect(Token::LBrace) {
            self.skip_to(Token::RBrace);
            return Err(e);
        }

        let mut arms = Vec::new();
        while self.tokens.peek().map(|t| &t.token) != Some(&Token::RBrace) {
            let pattern = match self.parse_pattern() {
                Ok(p) => p,
                Err(e) => {
                    self.errors.push(e);
                    self.recover_in_expr();
                    continue;
                }
            };
            
            let guard = if self.tokens.peek().map(|t| &t.token) == Some(&Token::If) {
                self.tokens.next();
                match self.parse_expr() {
                    Ok(g) => Some(g),
                    Err(e) => {
                        self.errors.push(e);
                        self.recover_in_expr();
                        None
                    }
                }
            } else {
                None
            };
            
            if let Err(e) = self.expect(Token::Arrow) {
                self.errors.push(e);
                self.recover_in_expr();
                continue;
            }
            
            let body = match self.parse_expr() {
                Ok(b) => b,
                Err(e) => {
                    self.errors.push(e);
                    self.recover_in_expr();
                    continue;
                }
            };
            
            if self.tokens.peek().map(|t| &t.token) != Some(&Token::Semicolon) {
                self.report_error(
                    ErrorKind::Parser,
                    "expected semicolon after match arm".to_string(),
                    self.current_span().unwrap_or(start_span)
                );
            } else {
                self.tokens.next();
            }

            arms.push(MatchArm {
                pattern,
                guard,
                body,
                span: start_span,
            });
        }

        self.expect(Token::RBrace)?;

        Ok(Stmt {
            kind: StmtKind::Match(MatchStmt {
                expr: Box::new(expr),
                arms,
                span: start_span,
            }),
            span: start_span,
        })
    }

    fn parse_expr(&mut self) -> Result<Expr, ChimError> {
        match self.parse_assign_expr() {
            Ok(expr) => Ok(expr),
            Err(e) => {
                self.recover_in_expr();
                Err(e)
            }
        }
    }

    fn parse_assign_expr(&mut self) -> Result<Expr, ChimError> {
        let left = self.parse_range_expr()?;

        match self.tokens.peek().map(|t| &t.token) {
            Some(&Token::Eq) => {
                self.tokens.next();
                let right = self.parse_assign_expr()?;
                Ok(Expr {
                    kind: ExprKind::Assign(AssignExpr {
                        left: Box::new(left),
                        right: Box::new(right),
                    }),
                    span: left.span,
                    ty: None,
                })
            }
            Some(&Token::PlusEq) | Some(&Token::MinusEq) | Some(&Token::StarEq) | Some(&Token::SlashEq)
            | Some(&Token::PercentEq) | Some(&Token::AndEq) | Some(&Token::PipeEq) | Some(&Token::CaretEq)
            | Some(&Token::LShiftEq) | Some(&Token::RShiftEq) => {
                let op = self.parse_assign_op()?;
                let right = self.parse_assign_expr()?;
                Ok(Expr {
                    kind: ExprKind::AssignOp(AssignOpExpr {
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                    }),
                    span: left.span,
                    ty: None,
                })
            }
            _ => Ok(left),
        }
    }

    fn parse_assign_op(&mut self) -> Result<BinOp, ChimError> {
        let op = match self.tokens.next().map(|t| &t.token) {
            Some(&Token::PlusEq) => BinOp::Add,
            Some(&Token::MinusEq) => BinOp::Sub,
            Some(&Token::StarEq) => BinOp::Mul,
            Some(&Token::SlashEq) => BinOp::Div,
            Some(&Token::PercentEq) => BinOp::Mod,
            Some(&Token::AndEq) => BinOp::BitAnd,
            Some(&Token::PipeEq) => BinOp::BitOr,
            Some(&Token::CaretEq) => BinOp::BitXor,
            Some(&Token::LShiftEq) => BinOp::Shl,
            Some(&Token::RShiftEq) => BinOp::Shr,
            _ => unreachable!(),
        };
        Ok(op)
    }

    fn parse_range_expr(&mut self) -> Result<Expr, ChimError> {
        let start = self.parse_logical_or_expr()?;

        match self.tokens.peek().map(|t| &t.token) {
            Some(Token::DotDot) | Some(Token::DotDotDot) => {
                let tok = self.tokens.next();
                let is_dotdotdot = match tok.map(|t| &t.token) {
                    Some(Token::DotDotDot) => true,
                    _ => false,
                };
                let end = if self.tokens.peek().map(|t| &t.token).map(|t| {
                    matches!(t, Token::LParen | Token::LBrace | Token::Identifier | Token::Int | Token::String)
                }).unwrap_or(false) {
                    Some(Box::new(self.parse_logical_or_expr()?))
                } else {
                    None
                };
                Ok(Expr {
                    kind: ExprKind::Range(RangeExpr {
                        start: Some(Box::new(start)),
                        end,
                        inclusive: is_dotdotdot,
                    }),
                    span: start.span,
                    ty: None,
                })
            }
            _ => Ok(start),
        }
    }

    fn parse_logical_or_expr(&mut self) -> Result<Expr, ChimError> {
        let mut left = self.parse_logical_and_expr()?;

        while let Some(Token::OrOr) = self.tokens.peek().map(|t| &t.token) {
            self.tokens.next();
            let right = self.parse_logical_and_expr()?;
            left = Expr {
                kind: ExprKind::Binary(BinaryExpr {
                    left: Box::new(left),
                    op: BinOp::Or,
                    right: Box::new(right),
                }),
                span: left.span,
                ty: None,
            };
        }

        Ok(left)
    }

    fn parse_logical_and_expr(&mut self) -> Result<Expr, ChimError> {
        let mut left = self.parse_bitwise_or_expr()?;

        while let Some(Token::AndAnd) = self.tokens.peek().map(|t| &t.token) {
            self.tokens.next();
            let right = self.parse_bitwise_or_expr()?;
            left = Expr {
                kind: ExprKind::Binary(BinaryExpr {
                    left: Box::new(left),
                    op: BinOp::And,
                    right: Box::new(right),
                }),
                span: left.span,
                ty: None,
            };
        }

        Ok(left)
    }

    fn parse_bitwise_or_expr(&mut self) -> Result<Expr, ChimError> {
        let mut left = self.parse_bitwise_xor_expr()?;

        while let Some(Token::Pipe) = self.tokens.peek().map(|t| &t.token) {
            self.tokens.next();
            let right = self.parse_bitwise_xor_expr()?;
            left = Expr {
                kind: ExprKind::Binary(BinaryExpr {
                    left: Box::new(left),
                    op: BinOp::BitOr,
                    right: Box::new(right),
                }),
                span: left.span,
                ty: None,
            };
        }

        Ok(left)
    }

    fn parse_bitwise_xor_expr(&mut self) -> Result<Expr, ChimError> {
        let mut left = self.parse_bitwise_and_expr()?;

        while let Some(Token::Caret) = self.tokens.peek().map(|t| &t.token) {
            self.tokens.next();
            let right = self.parse_bitwise_and_expr()?;
            left = Expr {
                kind: ExprKind::Binary(BinaryExpr {
                    left: Box::new(left),
                    op: BinOp::BitXor,
                    right: Box::new(right),
                }),
                span: left.span,
                ty: None,
            };
        }

        Ok(left)
    }

    fn parse_bitwise_and_expr(&mut self) -> Result<Expr, ChimError> {
        let mut left = self.parse_comparison_expr()?;

        while let Some(Token::Ampersand) = self.tokens.peek().map(|t| &t.token) {
            self.tokens.next();
            let right = self.parse_comparison_expr()?;
            left = Expr {
                kind: ExprKind::Binary(BinaryExpr {
                    left: Box::new(left),
                    op: BinOp::BitAnd,
                    right: Box::new(right),
                }),
                span: left.span,
                ty: None,
            };
        }

        Ok(left)
    }

    fn parse_comparison_expr(&mut self) -> Result<Expr, ChimError> {
        let left = self.parse_shift_expr()?;

        match self.tokens.peek().map(|t| &t.token) {
            Some(Token::EqEq) => {
                self.tokens.next();
                let right = self.parse_shift_expr()?;
                Ok(Expr {
                    kind: ExprKind::Binary(BinaryExpr {
                        left: Box::new(left),
                        op: BinOp::Eq,
                        right: Box::new(right),
                    }),
                    span: left.span,
                    ty: None,
                })
            }
            Some(Token::Neq) => {
                self.tokens.next();
                let right = self.parse_shift_expr()?;
                Ok(Expr {
                    kind: ExprKind::Binary(BinaryExpr {
                        left: Box::new(left),
                        op: BinOp::Ne,
                        right: Box::new(right),
                    }),
                    span: left.span,
                    ty: None,
                })
            }
            Some(Token::Lt) => {
                self.tokens.next();
                let right = self.parse_shift_expr()?;
                Ok(Expr {
                    kind: ExprKind::Binary(BinaryExpr {
                        left: Box::new(left),
                        op: BinOp::Lt,
                        right: Box::new(right),
                    }),
                    span: left.span,
                    ty: None,
                })
            }
            Some(Token::Lte) => {
                self.tokens.next();
                let right = self.parse_shift_expr()?;
                Ok(Expr {
                    kind: ExprKind::Binary(BinaryExpr {
                        left: Box::new(left),
                        op: BinOp::Le,
                        right: Box::new(right),
                    }),
                    span: left.span,
                    ty: None,
                })
            }
            Some(Token::Gt) => {
                self.tokens.next();
                let right = self.parse_shift_expr()?;
                Ok(Expr {
                    kind: ExprKind::Binary(BinaryExpr {
                        left: Box::new(left),
                        op: BinOp::Gt,
                        right: Box::new(right),
                    }),
                    span: left.span,
                    ty: None,
                })
            }
            Some(Token::Gte) => {
                self.tokens.next();
                let right = self.parse_shift_expr()?;
                Ok(Expr {
                    kind: ExprKind::Binary(BinaryExpr {
                        left: Box::new(left),
                        op: BinOp::Ge,
                        right: Box::new(right),
                    }),
                    span: left.span,
                    ty: None,
                })
            }
            _ => Ok(left),
        }
    }

    fn parse_shift_expr(&mut self) -> Result<Expr, ChimError> {
        let mut left = self.parse_additive_expr()?;

        while let Some(Token::LShift) = self.tokens.peek().map(|t| &t.token) {
            self.tokens.next();
            let right = self.parse_additive_expr()?;
            left = Expr {
                kind: ExprKind::Binary(BinaryExpr {
                    left: Box::new(left),
                    op: BinOp::Shl,
                    right: Box::new(right),
                }),
                span: left.span,
                ty: None,
            };
        }

        while let Some(Token::RShift) = self.tokens.peek().map(|t| &t.token) {
            self.tokens.next();
            let right = self.parse_additive_expr()?;
            left = Expr {
                kind: ExprKind::Binary(BinaryExpr {
                    left: Box::new(left),
                    op: BinOp::Shr,
                    right: Box::new(right),
                }),
                span: left.span,
                ty: None,
            };
        }

        Ok(left)
    }

    fn parse_additive_expr(&mut self) -> Result<Expr, ChimError> {
        let mut left = self.parse_multiplicative_expr()?;

        loop {
            match self.tokens.peek().map(|t| &t.token) {
                Some(&Token::Plus) => {
                    self.tokens.next();
                    let right = self.parse_multiplicative_expr()?;
                    left = Expr {
                        kind: ExprKind::Binary(BinaryExpr {
                            left: Box::new(left),
                            op: BinOp::Add,
                            right: Box::new(right),
                        }),
                        span: left.span,
                        ty: None,
                    };
                }
                Some(&Token::Minus) => {
                    self.tokens.next();
                    let right = self.parse_multiplicative_expr()?;
                    left = Expr {
                        kind: ExprKind::Binary(BinaryExpr {
                            left: Box::new(left),
                            op: BinOp::Sub,
                            right: Box::new(right),
                        }),
                        span: left.span,
                        ty: None,
                    };
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_multiplicative_expr(&mut self) -> Result<Expr, ChimError> {
        let mut left = self.parse_unary_expr()?;

        loop {
            match self.tokens.peek().map(|t| &t.token) {
                Some(&Token::Star) => {
                    self.tokens.next();
                    let right = self.parse_unary_expr()?;
                    left = Expr {
                        kind: ExprKind::Binary(BinaryExpr {
                            left: Box::new(left),
                            op: BinOp::Mul,
                            right: Box::new(right),
                        }),
                        span: left.span,
                        ty: None,
                    };
                }
                Some(&Token::Slash) => {
                    self.tokens.next();
                    let right = self.parse_unary_expr()?;
                    left = Expr {
                        kind: ExprKind::Binary(BinaryExpr {
                            left: Box::new(left),
                            op: BinOp::Div,
                            right: Box::new(right),
                        }),
                        span: left.span,
                        ty: None,
                    };
                }
                Some(&Token::Percent) => {
                    self.tokens.next();
                    let right = self.parse_unary_expr()?;
                    left = Expr {
                        kind: ExprKind::Binary(BinaryExpr {
                            left: Box::new(left),
                            op: BinOp::Mod,
                            right: Box::new(right),
                        }),
                        span: left.span,
                        ty: None,
                    };
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr, ChimError> {
        match self.tokens.peek().map(|t| &t.token) {
            Some(&Token::Minus) => {
                self.tokens.next();
                let expr = self.parse_unary_expr()?;
                Ok(Expr {
                    kind: ExprKind::Unary(UnaryExpr {
                        op: UnOp::Neg,
                        expr: Box::new(expr),
                    }),
                    span: expr.span,
                    ty: None,
                })
            }
            Some(&Token::Bang) => {
                self.tokens.next();
                let expr = self.parse_unary_expr()?;
                Ok(Expr {
                    kind: ExprKind::Unary(UnaryExpr {
                        op: UnOp::Not,
                        expr: Box::new(expr),
                    }),
                    span: expr.span,
                    ty: None,
                })
            }
            Some(&Token::Ampersand) => {
                self.tokens.next();
                let mutability = if self.tokens.peek().map(|t| &t.token) == Some(&Token::Mut) {
                    self.tokens.next();
                    Mutability::Mutable
                } else {
                    Mutability::Immutable
                };
                let expr = self.parse_unary_expr()?;
                Ok(Expr {
                    kind: ExprKind::Unary(UnaryExpr {
                        op: if mutability == Mutability::Mutable { UnOp::RefMut } else { UnOp::Ref },
                        expr: Box::new(expr),
                    }),
                    span: expr.span,
                    ty: None,
                })
            }
            Some(&Token::Star) => {
                self.tokens.next();
                let expr = self.parse_unary_expr()?;
                Ok(Expr {
                    kind: ExprKind::Unary(UnaryExpr {
                        op: UnOp::Deref,
                        expr: Box::new(expr),
                    }),
                    span: expr.span,
                    ty: None,
                })
            }
            _ => self.parse_postfix_expr(),
        }
    }

    fn parse_postfix_expr(&mut self) -> Result<Expr, ChimError> {
        let mut expr = self.parse_primary_expr()?;

        loop {
            match self.tokens.peek().map(|t| &t.token) {
                Some(&Token::Dot) => {
                    self.tokens.next();
                    let field = self.parse_identifier()?;
                    let span = expr.span;
                    expr = Expr {
                        kind: ExprKind::FieldAccess(FieldAccessExpr {
                            expr: Box::new(expr),
                            field: Arc::from(field),
                        }),
                        span,
                        ty: None,
                    };
                }
                Some(&Token::LParen) => {
                    self.tokens.next();
                    let mut args = SmallVec::new();
                    while self.tokens.peek().map(|t| &t.token) != Some(&Token::RParen) {
                        args.push(self.parse_expr()?);
                        if self.tokens.peek().map(|t| &t.token) == Some(&Token::Comma) {
                            self.tokens.next();
                        }
                    }
                    self.expect(Token::RParen)?;
                    expr = Expr {
                        kind: ExprKind::Call(CallExpr {
                            func: Box::new(expr),
                            args,
                        }),
                        span: expr.span,
                        ty: None,
                    };
                }
                Some(&Token::LBracket) => {
                    self.tokens.next();
                    let index = self.parse_expr()?;
                    self.expect(Token::RBracket)?;
                    expr = Expr {
                        kind: ExprKind::Index(IndexExpr {
                            expr: Box::new(expr),
                            index: Box::new(index),
                        }),
                        span: expr.span,
                        ty: None,
                    };
                }
                Some(&Token::As) => {
                    self.tokens.next();
                    let ty = self.parse_type()?;
                    expr = Expr {
                        kind: ExprKind::Cast(CastExpr {
                            expr: Box::new(expr),
                            ty,
                        }),
                        span: expr.span,
                        ty: None,
                    };
                }
                Some(&Token::Question) => {
                    self.tokens.next();
                    let then_branch = self.parse_expr()?;
                    self.expect(Token::Colon)?;
                    let else_branch = self.parse_expr()?;
                    expr = Expr {
                        kind: ExprKind::Ternary(TernaryExpr {
                            condition: Box::new(expr),
                            then_branch: Box::new(then_branch),
                            else_branch: Box::new(else_branch),
                        }),
                        span: expr.span,
                        ty: None,
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary_expr(&mut self) -> Result<Expr, ChimError> {
        let start_span = self.current_span()?;

        match self.tokens.next().map(|t| &t.token) {
            Some(Token::Identifier) => {
                let name = self.intern_identifier();
                let span = start_span;
                Ok(Expr {
                    kind: ExprKind::Identifier(Arc::from(name)),
                    span,
                    ty: None,
                })
            }
            Some(Token::Int) => {
                let text = self.intern_string();
                let value = text.parse::<i128>().unwrap_or(0);
                Ok(Expr {
                    kind: ExprKind::Literal(Literal {
                        kind: LiteralKind::Int(value),
                        span: start_span,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Float) => {
                let text = self.intern_string();
                let value = text.parse::<f64>().unwrap_or(0.0);
                Ok(Expr {
                    kind: ExprKind::Literal(Literal {
                        kind: LiteralKind::Float(value),
                        span: start_span,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::String) | Some(Token::RawString) => {
                let text = Arc::from(self.intern_string());
                Ok(Expr {
                    kind: ExprKind::Literal(Literal {
                        kind: LiteralKind::String(text),
                        span: start_span,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::ByteString) => {
                let text = Arc::from(self.intern_string());
                Ok(Expr {
                    kind: ExprKind::Literal(Literal {
                        kind: LiteralKind::ByteString(text),
                        span: start_span,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Char) => {
                let text = self.intern_string();
                let value = text.chars().next().unwrap_or('\0');
                Ok(Expr {
                    kind: ExprKind::Literal(Literal {
                        kind: LiteralKind::Char(value),
                        span: start_span,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Byte) => {
                let text = self.intern_string();
                let value = text.parse::<u8>().unwrap_or(0);
                Ok(Expr {
                    kind: ExprKind::Literal(Literal {
                        kind: LiteralKind::Byte(value),
                        span: start_span,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::True) => {
                Ok(Expr {
                    kind: ExprKind::Literal(Literal {
                        kind: LiteralKind::Bool(true),
                        span: start_span,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::False) => {
                Ok(Expr {
                    kind: ExprKind::Literal(Literal {
                        kind: LiteralKind::Bool(false),
                        span: start_span,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Null) => {
                Ok(Expr {
                    kind: ExprKind::Literal(Literal {
                        kind: LiteralKind::Null,
                        span: start_span,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Unit) => {
                Ok(Expr {
                    kind: ExprKind::Literal(Literal {
                        kind: LiteralKind::Unit,
                        span: start_span,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::LParen) => {
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            Some(Token::LBracket) => {
                let mut elements = SmallVec::new();
                while self.tokens.peek().map(|t| &t.token) != Some(&Token::RBracket) {
                    elements.push(self.parse_expr()?);
                    if self.tokens.peek().map(|t| &t.token) == Some(&Token::Comma) {
                        self.tokens.next();
                    }
                }
                self.expect(Token::RBracket)?;
                Ok(Expr {
                    kind: ExprKind::Array(ArrayExpr { elements }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::LBrace) => {
                let stmts = self.parse_block()?;
                Ok(Expr {
                    kind: ExprKind::Block(BlockExpr {
                        label: None,
                        stmts,
                        ty: None,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::If) => {
                let condition = self.parse_expr()?;
                let then_branch = BlockExpr {
                    label: None,
                    stmts: self.parse_block()?,
                    ty: None,
                };
                let else_branch = if self.tokens.peek().map(|t| &t.token) == Some(&Token::Else) {
                    self.tokens.next();
                    Some(Box::new(self.parse_expr()?))
                } else {
                    None
                };
                Ok(Expr {
                    kind: ExprKind::If(IfExpr {
                        condition: Box::new(condition),
                        then_branch,
                        else_branch,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Match) => {
                let expr = self.parse_expr()?;
                self.expect(Token::LBrace)?;
                let mut arms = Vec::new();
                while self.tokens.peek().map(|t| &t.token) != Some(&Token::RBrace) {
                    let pattern = self.parse_pattern()?;
                    let guard = if self.tokens.peek().map(|t| &t.token) == Some(&Token::If) {
                        self.tokens.next();
                        Some(self.parse_expr()?)
                    } else {
                        None
                    };
                    self.expect(Token::Arrow)?;
                    let body = self.parse_expr()?;
                    arms.push(MatchArm {
                        pattern,
                        guard,
                        body,
                        span: start_span,
                    });
                    if self.tokens.peek().map(|t| &t.token) == Some(&Token::Comma) {
                        self.tokens.next();
                    }
                }
                self.expect(Token::RBrace)?;
                Ok(Expr {
                    kind: ExprKind::Match(MatchExpr {
                        expr: Box::new(expr),
                        arms,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Pipe) => {
                let _is_move = false;
                let mut params = Vec::new();
                if self.tokens.peek().map(|t| &t.token) != Some(&Token::Pipe) {
                    loop {
                        let name = self.parse_identifier()?;
                        let ty = if self.tokens.peek().map(|t| &t.token) == Some(&Token::Colon) {
                            self.tokens.next();
                            Some(self.parse_type()?)
                        } else {
                            None
                        };
                        params.push(Param {
                            name: Arc::from(name),
                            ty: ty.unwrap_or(Type {
                                kind: Box::new(TypeKind::Infer),
                                span: start_span,
                            }),
                            span: start_span,
                            is_mut: false,
                            is_ref: false,
                        });
                        if self.tokens.peek().map(|t| &t.token) == Some(&Token::Comma) {
                            self.tokens.next();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(Token::Pipe)?;
                let body = Box::new(self.parse_expr()?);
                Ok(Expr {
                    kind: ExprKind::Closure(ClosureExpr {
                        params,
                        body,
                        is_async: false,
                        is_move: _is_move,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Wait) => {
                let atomic = self.parse_expr()?;
                let timeout = if self.tokens.peek().map(|t| &t.token) == Some(&Token::Comma) {
                    self.tokens.next();
                    Some(Box::new(self.parse_expr()?))
                } else {
                    None
                };
                Ok(Expr {
                    kind: ExprKind::Wait(WaitExpr {
                        atomic: Box::new(atomic),
                        timeout,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Notify) => {
                let atomic = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Notify(NotifyExpr {
                        atomic: Box::new(atomic),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::NotifyAll) => {
                let atomic = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::NotifyAll(NotifyAllExpr {
                        atomic: Box::new(atomic),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Iterator) => {
                let iterable = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Iterator(IteratorExpr {
                        iterable: Box::new(iterable),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Next) => {
                let iterator = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Next(NextExpr {
                        iterator: Box::new(iterator),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Item) => {
                let iterator = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Item(ItemExpr {
                        iterator: Box::new(iterator),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Collect) => {
                let iterator = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Collect(CollectExpr {
                        iterator: Box::new(iterator),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Chain) => {
                let iterator1 = self.parse_expr()?;
                self.expect(Token::Comma)?;
                let iterator2 = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Chain(ChainExpr {
                        iterator1: Box::new(iterator1),
                        iterator2: Box::new(iterator2),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Filter) => {
                let iterator = self.parse_expr()?;
                self.expect(Token::Comma)?;
                let predicate = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Filter(FilterExpr {
                        iterator: Box::new(iterator),
                        predicate: Box::new(predicate),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Fold) => {
                let iterator = self.parse_expr()?;
                self.expect(Token::Comma)?;
                let init = self.parse_expr()?;
                self.expect(Token::Comma)?;
                let accumulator = self.parse_identifier()?;
                self.expect(Token::Comma)?;
                let body = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Fold(FoldExpr {
                        iterator: Box::new(iterator),
                        init: Box::new(init),
                        accumulator: Arc::from(accumulator),
                        body: Box::new(body),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Map) => {
                let iterator = self.parse_expr()?;
                self.expect(Token::Comma)?;
                let mapper = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Map(MapExpr {
                        iterator: Box::new(iterator),
                        mapper: Box::new(mapper),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Result) => {
                self.expect(Token::LAngle)?;
                let ok_type = self.parse_type()?;
                self.expect(Token::Comma)?;
                let err_type = self.parse_type()?;
                self.expect(Token::RAngle)?;
                Ok(Expr {
                    kind: ExprKind::Result(ResultExpr {
                        ok_type,
                        err_type,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Ok) => {
                let value = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Ok(OkExpr {
                        value: Box::new(value),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Err) => {
                let error = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Err(ErrExpr {
                        error: Box::new(error),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Try) => {
                let expr = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Try(TryExpr {
                        expr: Box::new(expr),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Catch) => {
                let try_expr = self.parse_expr()?;
                self.expect(Token::Comma)?;
                let error_var = self.parse_identifier()?;
                self.expect(Token::Comma)?;
                let catch_expr = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Catch(CatchExpr {
                        try_expr: Box::new(try_expr),
                        error_var: Arc::from(error_var),
                        catch_expr: Box::new(catch_expr),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Error) => {
                let message = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::ErrorExpr(ErrorExpr {
                        message: Box::new(message),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Context) => {
                let context = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Context(ContextExpr {
                        context: Box::new(context),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Throw) => {
                let error = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Throw(ThrowExpr {
                        error: Box::new(error),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Future) => {
                let body = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Future(FutureExpr {
                        body: Box::new(body),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Yield) => {
                let value = if self.tokens.peek().map(|t| &t.token).map(|t| {
                    matches!(t, Token::LParen | Token::LBrace | Token::Identifier | Token::Int | Token::String)
                }).unwrap_or(false) {
                    Some(Box::new(self.parse_expr()?))
                } else {
                    None
                };
                Ok(Expr {
                    kind: ExprKind::Yield(YieldExpr {
                        value,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Stream) => {
                let body = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Stream(StreamExpr {
                        body: Box::new(body),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Unsafe) => {
                let body = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Unsafe(UnsafeExpr {
                        body: Box::new(body),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Alloc) => {
                let ty = self.parse_type()?;
                let size = if self.tokens.peek().map(|t| &t.token) == Some(&Token::LParen) {
                    self.tokens.next();
                    Some(Box::new(self.parse_expr()?))
                } else {
                    None
                };
                Ok(Expr {
                    kind: ExprKind::Alloc(AllocExpr {
                        ty,
                        size,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::AllocAligned) => {
                let ty = self.parse_type()?;
                self.expect(Token::LParen)?;
                let size = Box::new(self.parse_expr()?);
                self.expect(Token::Comma)?;
                let alignment = Box::new(self.parse_expr()?);
                self.expect(Token::RParen)?;
                Ok(Expr {
                    kind: ExprKind::AllocAligned(AllocAlignedExpr {
                        ty,
                        size,
                        alignment,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Free) => {
                let ptr = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Free(FreeExpr {
                        ptr: Box::new(ptr),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Ptr) => {
                let ty = self.parse_type()?;
                Ok(Expr {
                    kind: ExprKind::Ptr(PtrExpr {
                        ty,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::PtrAdd) => {
                let ptr = self.parse_expr()?;
                self.expect(Token::Comma)?;
                let offset = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::PtrAdd(PtrAddExpr {
                        ptr: Box::new(ptr),
                        offset: Box::new(offset),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::PtrSub) => {
                let ptr1 = self.parse_expr()?;
                self.expect(Token::Comma)?;
                let ptr2 = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::PtrSub(PtrSubExpr {
                        ptr1: Box::new(ptr1),
                        ptr2: Box::new(ptr2),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::PtrLoad) => {
                let ptr = self.parse_expr()?;
                let ty = self.parse_type()?;
                Ok(Expr {
                    kind: ExprKind::PtrLoad(PtrLoadExpr {
                        ptr: Box::new(ptr),
                        ty,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::PtrStore) => {
                let ptr = self.parse_expr()?;
                self.expect(Token::Comma)?;
                let value = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::PtrStore(PtrStoreExpr {
                        ptr: Box::new(ptr),
                        value: Box::new(value),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::PtrCast) => {
                let ptr = self.parse_expr()?;
                self.expect(Token::As)?;
                let target_ty = self.parse_type()?;
                Ok(Expr {
                    kind: ExprKind::PtrCast(PtrCastExpr {
                        ptr: Box::new(ptr),
                        target_ty,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::PtrOffsetOf) => {
                let ptr = self.parse_expr()?;
                self.expect(Token::Comma)?;
                let field = self.parse_identifier()?;
                Ok(Expr {
                    kind: ExprKind::PtrOffsetOf(PtrOffsetOfExpr {
                        ptr: Box::new(ptr),
                        field: Arc::from(field),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::PtrSizeOf) => {
                self.expect(Token::LParen)?;
                let ty = self.parse_type()?;
                self.expect(Token::RParen)?;
                Ok(Expr {
                    kind: ExprKind::PtrSizeOf(PtrSizeOfExpr {
                        ty,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::AlignOf) => {
                self.expect(Token::LParen)?;
                let ty = self.parse_type()?;
                self.expect(Token::RParen)?;
                Ok(Expr {
                    kind: ExprKind::AlignOf(AlignOfExpr {
                        ty,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Proof) => {
                let proposition = self.parse_expr()?;
                self.expect(Token::Colon)?;
                let proof = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Proof(ProofExpr {
                        proposition: Box::new(proposition),
                        proof: Box::new(proof),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Theorem) => {
                let name = self.parse_identifier()?;
                let params = self.parse_function_params()?;
                self.expect(Token::Colon)?;
                let proposition = self.parse_expr()?;
                self.expect(Token::Eq)?;
                let proof = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Theorem(TheoremExpr {
                        name: Arc::from(name),
                        params,
                        proposition: Box::new(proposition),
                        proof: Box::new(proof),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Lemma) => {
                let name = self.parse_identifier()?;
                let params = self.parse_function_params()?;
                self.expect(Token::Colon)?;
                let proposition = self.parse_expr()?;
                self.expect(Token::Eq)?;
                let proof = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Lemma(LemmaExpr {
                        name: Arc::from(name),
                        params,
                        proposition: Box::new(proposition),
                        proof: Box::new(proof),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Induction) => {
                let variable = self.parse_identifier()?;
                self.expect(Token::Colon)?;
                let base_case = self.parse_expr()?;
                self.expect(Token::Comma)?;
                let inductive_step = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Induction(InductionExpr {
                        variable: Arc::from(variable),
                        base_case: Box::new(base_case),
                        inductive_step: Box::new(inductive_step),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Case) => {
                let value = self.parse_expr()?;
                self.expect(Token::Colon)?;
                let cases = self.parse_match_cases()?;
                Ok(Expr {
                    kind: ExprKind::Case(CaseExpr {
                        value: Box::new(value),
                        cases,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Refl) => {
                let ty = self.parse_type()?;
                Ok(Expr {
                    kind: ExprKind::Refl(ReflExpr {
                        ty,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Cong) => {
                let ty = self.parse_type()?;
                self.expect(Token::LParen)?;
                let expr1 = self.parse_expr()?;
                self.expect(Token::Comma)?;
                let expr2 = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(Expr {
                    kind: ExprKind::Cong(CongExpr {
                        ty,
                        expr1: Box::new(expr1),
                        expr2: Box::new(expr2),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Sym) => {
                let ty = self.parse_type()?;
                self.expect(Token::LParen)?;
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(Expr {
                    kind: ExprKind::Sym(SymExpr {
                        ty,
                        expr: Box::new(expr),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Trans) => {
                let ty = self.parse_type()?;
                self.expect(Token::LParen)?;
                let expr1 = self.parse_expr()?;
                self.expect(Token::Comma)?;
                let expr2 = self.parse_expr()?;
                self.expect(Token::Comma)?;
                let expr3 = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(Expr {
                    kind: ExprKind::Trans(TransExpr {
                        ty,
                        expr1: Box::new(expr1),
                        expr2: Box::new(expr2),
                        expr3: Box::new(expr3),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Rec) => {
                let ty = self.parse_type()?;
                self.expect(Token::Colon)?;
                let body = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Rec(RecExpr {
                        ty,
                        body: Box::new(body),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Fix) => {
                let ty = self.parse_type()?;
                self.expect(Token::Colon)?;
                let body = self.parse_expr()?;
                Ok(Expr {
                    kind: ExprKind::Fix(FixExpr {
                        ty,
                        body: Box::new(body),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Class) => {
                let name = self.parse_identifier()?;
                let params = self.parse_function_params()?;
                let methods = self.parse_class_methods()?;
                Ok(Expr {
                    kind: ExprKind::Class(ClassExpr {
                        name: Arc::from(name),
                        params,
                        methods,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Instance) => {
                let class_name = self.parse_identifier()?;
                let ty = self.parse_type()?;
                let methods = self.parse_class_methods()?;
                Ok(Expr {
                    kind: ExprKind::Instance(InstanceExpr {
                        class_name: Arc::from(class_name),
                        ty,
                        methods,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Where) => {
                let expr = self.parse_expr()?;
                self.expect(Token::LBrace)?;
                let mut constraints = Vec::new();
                while self.tokens.peek().map(|t| &t.token) != Some(&Token::RBrace) {
                    constraints.push(self.parse_expr()?);
                    if self.tokens.peek().map(|t| &t.token) == Some(&Token::Comma) {
                        self.tokens.next();
                    }
                }
                self.expect(Token::RBrace)?;
                Ok(Expr {
                    kind: ExprKind::Where(WhereExpr {
                        expr: Box::new(expr),
                        constraints,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::EqProp) => {
                let ty = self.parse_type()?;
                self.expect(Token::LParen)?;
                let left = self.parse_expr()?;
                self.expect(Token::Eq)?;
                let right = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(Expr {
                    kind: ExprKind::EqProp(EqPropExpr {
                        ty,
                        left: Box::new(left),
                        right: Box::new(right),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::ReflProp) => {
                let ty = self.parse_type()?;
                self.expect(Token::LParen)?;
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(Expr {
                    kind: ExprKind::ReflProp(ReflPropExpr {
                        ty,
                        expr: Box::new(expr),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::JMeq) => {
                let ty = self.parse_type()?;
                self.expect(Token::LParen)?;
                let expr1 = self.parse_expr()?;
                self.expect(Token::Comma)?;
                let expr2 = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(Expr {
                    kind: ExprKind::JMeq(JMeqExpr {
                        ty,
                        expr1: Box::new(expr1),
                        expr2: Box::new(expr2),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::Rewrite) => {
                let ty = self.parse_type()?;
                self.expect(Token::LParen)?;
                let expr = self.parse_expr()?;
                self.expect(Token::Comma)?;
                let rule = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(Expr {
                    kind: ExprKind::Rewrite(RewriteExpr {
                        ty,
                        expr: Box::new(expr),
                        rule: Box::new(rule),
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            Some(Token::With) => {
                let expr = self.parse_expr()?;
                self.expect(Token::LBrace)?;
                let mut bindings = Vec::new();
                while self.tokens.peek().map(|t| &t.token) != Some(&Token::RBrace) {
                    let name = self.parse_identifier()?;
                    self.expect(Token::Eq)?;
                    let value = self.parse_expr()?;
                    bindings.push((Arc::from(name), value));
                    if self.tokens.peek().map(|t| &t.token) == Some(&Token::Comma) {
                        self.tokens.next();
                    }
                }
                self.expect(Token::RBrace)?;
                Ok(Expr {
                    kind: ExprKind::With(WithExpr {
                        expr: Box::new(expr),
                        bindings,
                    }),
                    span: start_span,
                    ty: None,
                })
            }
            _ => {
                self.errors.push(ChimError::new(
                    ErrorKind::Parser,
                    "invalid expression".to_string(),
                ).with_span(start_span));
                Err(self.errors.last().unwrap().clone())
            }
        }
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ChimError> {
        let start_span = self.current_span()?;

        match self.tokens.peek().map(|t| &t.token) {
            Some(Token::Identifier) => {
                let name = self.parse_identifier()?;
                Ok(Pattern {
                    kind: PatternKind::Identifier(Arc::from(name)),
                    span: start_span,
                })
            }
            Some(Token::Underscore) => {
                self.tokens.next();
                Ok(Pattern {
                    kind: PatternKind::Wildcard,
                    span: start_span,
                })
            }
            Some(Token::LParen) => {
                self.tokens.next();
                let mut patterns = Vec::new();
                while self.tokens.peek().map(|t| &t.token) != Some(&Token::RParen) {
                    patterns.push(self.parse_pattern()?);
                    if self.tokens.peek().map(|t| &t.token) == Some(&Token::Comma) {
                        self.tokens.next();
                    }
                }
                self.expect(Token::RParen)?;
                Ok(Pattern {
                    kind: PatternKind::Tuple(patterns),
                    span: start_span,
                })
            }
            Some(Token::LBracket) => {
                self.tokens.next();
                let mut patterns = Vec::new();
                while self.tokens.peek().map(|t| &t.token) != Some(&Token::RBracket) {
                    patterns.push(self.parse_pattern()?);
                    if self.tokens.peek().map(|t| &t.token) == Some(&Token::Comma) {
                        self.tokens.next();
                    }
                }
                self.expect(Token::RBracket)?;
                Ok(Pattern {
                    kind: PatternKind::Slice(patterns),
                    span: start_span,
                })
            }
            _ => Ok(Pattern {
                kind: PatternKind::Wildcard,
                span: start_span,
            }),
        }
    }

    fn parse_type(&mut self) -> Result<Type, ChimError> {
        let start_span = self.current_span()?;

        match self.tokens.peek().map(|t| &t.token) {
            Some(Token::LParen) => {
                self.tokens.next();
                let mut types = Vec::new();
                while self.tokens.peek().map(|t| &t.token) != Some(&Token::RParen) {
                    types.push(self.parse_type()?);
                    if self.tokens.peek().map(|t| &t.token) == Some(&Token::Comma) {
                        self.tokens.next();
                    }
                }
                self.expect(Token::RParen)?;
                Ok(Type {
                    kind: Box::new(TypeKind::Tuple(types)),
                    span: start_span,
                })
            }
            Some(Token::LBracket) => {
                self.tokens.next();
                let inner = self.parse_type()?;
                if self.tokens.peek().map(|t| &t.token) == Some(&Token::Semicolon) {
                    self.tokens.next();
                    if let Some(Token::Int) = self.tokens.peek().map(|t| &t.token) {
                        let size = self.intern_string().parse::<usize>().unwrap_or(0);
                        self.tokens.next();
                        self.expect(Token::RBracket)?;
                        return Ok(Type {
                            kind: Box::new(TypeKind::Array(Box::new(inner), size)),
                            span: start_span,
                        });
                    }
                }
                self.expect(Token::RBracket)?;
                Ok(Type {
                    kind: Box::new(TypeKind::Slice(Box::new(inner))),
                    span: start_span,
                })
            }
            Some(Token::Star) => {
                self.tokens.next();
                let ty = self.parse_type()?;
                Ok(Type {
                    kind: Box::new(TypeKind::Pointer(Box::new(ty), Mutability::Immutable)),
                    span: start_span,
                })
            }
            Some(Token::Ampersand) => {
                self.tokens.next();
                let lifetime = if let Some(Token::Identifier) = self.tokens.peek().map(|t| &t.token) {
                    let name = self.parse_identifier()?;
                    Some(Lifetime {
                        name: Arc::from(name),
                        span: start_span,
                    })
                } else {
                    None
                };
                let mutability = if self.tokens.peek().map(|t| &t.token) == Some(&Token::Mut) {
                    self.tokens.next();
                    Mutability::Mutable
                } else {
                    Mutability::Immutable
                };
                let ty = self.parse_type()?;
                Ok(Type {
                    kind: Box::new(TypeKind::Reference(lifetime, Box::new(ty), mutability)),
                    span: start_span,
                })
            }
            Some(Token::Func) => {
                self.tokens.next();
                self.expect(Token::LParen)?;
                let mut params = Vec::new();
                while self.tokens.peek().map(|t| &t.token) != Some(&Token::RParen) {
                    params.push(self.parse_type()?);
                    if self.tokens.peek().map(|t| &t.token) == Some(&Token::Comma) {
                        self.tokens.next();
                    }
                }
                self.expect(Token::RParen)?;
                let return_type = if self.tokens.peek().map(|t| &t.token) == Some(&Token::ThinArrow) {
                    self.tokens.next();
                    Some(self.parse_type()?)
                } else {
                    None
                };
                Ok(Type {
                    kind: Box::new(TypeKind::Function(FunctionType {
                        params,
                        return_type: Box::new(return_type.unwrap_or(Type {
                            kind: Box::new(TypeKind::Tuple(Vec::new())),
                            span: start_span,
                        })),
                        is_async: false,
                    })),
                    span: start_span,
                })
            }
            Some(Token::PathSep) => {
                self.tokens.next();
                let name = self.parse_identifier()?;
                Ok(Type {
                    kind: Box::new(TypeKind::Path(Path {
                        segments: vec![PathSegment {
                            ident: Arc::from(name),
                            args: Vec::new(),
                            span: start_span,
                        }],
                        span: start_span,
                    })),
                    span: start_span,
                })
            }
            _ => {
                let path = self.parse_path()?;
                Ok(Type {
                    kind: Box::new(TypeKind::Path(path)),
                    span: start_span,
                })
            }
        }
    }

    fn parse_class_methods(&mut self) -> Result<Vec<Function>, ChimError> {
        let mut methods = Vec::new();
        while self.tokens.peek().map(|t| &t.token) != Some(&Token::RBrace) {
            let method = self.parse_function()?;
            methods.push(method);
        }
        self.expect(Token::RBrace)?;
        Ok(methods)
    }

    fn parse_match_cases(&mut self) -> Result<Vec<MatchCase>, ChimError> {
        let mut cases = Vec::new();
        while self.tokens.peek().map(|t| &t.token) != Some(&Token::RBrace) {
            let pattern = self.parse_pattern()?;
            self.expect(Token::Arrow)?;
            let body = self.parse_expr()?;
            cases.push(MatchCase {
                pattern,
                body,
            });
            if self.tokens.peek().map(|t| &t.token) == Some(&Token::Comma) {
                self.tokens.next();
            }
        }
        self.expect(Token::RBrace)?;
        Ok(cases)
    }

    fn parse_path(&mut self) -> Result<Path, ChimError> {
        let start_span = self.current_span()?;
        let mut segments = Vec::new();

        loop {
            let segment_start = self.current_span()?;
            let ident = self.parse_identifier()?;
            segments.push(PathSegment {
                ident: Arc::from(ident),
                args: Vec::new(),
                span: segment_start,
            });

            if self.tokens.peek().map(|t| &t.token) == Some(&Token::PathSep) {
                self.tokens.next();
            } else {
                break;
            }
        }

        Ok(Path {
            segments,
            span: start_span,
        })
    }

    fn parse_optional_type_path(&mut self) -> Result<Option<Type>, ChimError> {
        if self.tokens.peek().map(|t| &t.token) == Some(&Token::LParen) {
            return Ok(None);
        }
        self.parse_type().map(Some)
    }

    fn parse_generics(&mut self) -> Result<Vec<LifetimeParam>, ChimError> {
        if self.tokens.peek().map(|t| &t.token) != Some(&Token::LAngle) {
            return Ok(Vec::new());
        }
        self.tokens.next();
        let mut lifetimes = Vec::new();

        while self.tokens.peek().map(|t| &t.token) != Some(&Token::RAngle) {
            if let Some(Token::Identifier) = self.tokens.peek().map(|t| &t.token) {
                let name = self.parse_identifier()?;
                lifetimes.push(LifetimeParam {
                    name: Arc::from(name),
                    bounds: Vec::new(),
                    span: Span::new(self.file_id, 0, 0, 0, 0),
                });
            }
            if self.tokens.peek().map(|t| &t.token) == Some(&Token::Comma) {
                self.tokens.next();
            }
        }

        self.expect(Token::RAngle)?;
        Ok(lifetimes)
    }

    fn parse_where_clauses(&mut self) -> Result<Vec<WhereClause>, ChimError> {
        if self.tokens.peek().map(|t| &t.token) != Some(&Token::Where) {
            return Ok(Vec::new());
        }
        self.tokens.next();
        let mut predicates = Vec::new();

        while !matches!(self.tokens.peek().map(|t| &t.token), Some(&Token::LBrace) | None) {
            let bounded_type = self.parse_type()?;
            predicates.push(WherePredicate {
                bounded_type,
                bounds: Vec::new(),
                span: Span::new(self.file_id, 0, 0, 0, 0),
            });
            if self.tokens.peek().map(|t| &t.token) == Some(&Token::Comma) {
                self.tokens.next();
            }
        }

        Ok(vec![WhereClause {
            predicates,
            span: Span::new(self.file_id, 0, 0, 0, 0),
        }])
    }

    fn parse_trait_bounds(&mut self) -> Result<Vec<Type>, ChimError> {
        let mut bounds = Vec::new();

        while self.tokens.peek().map(|t| &t.token) != Some(&Token::LBrace)
            && self.tokens.peek().map(|t| &t.token) != Some(&Token::Where)
            && !matches!(self.tokens.peek().map(|t| &t.token), Some(&Token::LParen) | None)
        {
            bounds.push(self.parse_type()?);
        }

        Ok(bounds)
    }

    fn parse_visibility(&mut self) -> Result<bool, ChimError> {
        match self.tokens.peek().map(|t| &t.token) {
            Some(Token::Pub) => {
                self.tokens.next();
                Ok(true)
            }
            Some(Token::Priv) => {
                self.tokens.next();
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    fn parse_identifier(&mut self) -> Result<String, ChimError> {
        match self.tokens.next().map(|t| &t.token) {
            Some(Token::Identifier) => Ok(self.intern_identifier()),
            _ => {
                self.errors.push(ChimError::new(
                    ErrorKind::Parser,
                    "expected identifier".to_string(),
                ).with_span(self.current_span()?));
                let err = self.errors.last().cloned().unwrap_or_else(|| {
                    ChimError::new(ErrorKind::Parser, "unknown error".to_string())
                });
                Err(err)
            }
        }
    }

    fn parse_int_literal(&mut self) -> Result<i128, ChimError> {
        if let Some(Token::Int) = self.tokens.next().map(|t| &t.token) {
            let text = self.intern_string();
            Ok(self.parse_base_literal(&text))
        } else {
            Err(ChimError::new(
                ErrorKind::Parser,
                "expected integer literal".to_string(),
            ).with_span(self.current_span()?))
        }
    }

    fn parse_base_literal(&self, text: &str) -> i128 {
        if text.starts_with("0x") || text.starts_with("0X") {
            i128::from_str_radix(&text[2..].replace('_', ""), 16).unwrap_or(0)
        } else if text.starts_with("0b") || text.starts_with("0B") {
            i128::from_str_radix(&text[2..].replace('_', ""), 2).unwrap_or(0)
        } else if text.starts_with("0o") || text.starts_with("0O") {
            i128::from_str_radix(&text[2..].replace('_', ""), 8).unwrap_or(0)
        } else if text.starts_with("0t") || text.starts_with("0T") {
            i128::from_str_radix(&text[2..].replace('_', ""), 3).unwrap_or(0)
        } else if text.starts_with("0e") || text.starts_with("0E") {
            self.parse_balanced_ternary(&text[2..].replace('_', ""))
        } else if text.starts_with("0d") || text.starts_with("0D") {
            self.parse_duodecimal(&text[2..].replace('_', ""))
        } else if text.starts_with("0h") || text.starts_with("0H") {
            self.parse_tetravigesimal(&text[2..].replace('_', ""))
        } else if text.starts_with("0s") || text.starts_with("0S") {
            self.parse_sexagesimal(&text[2..].replace('_', ""))
        } else {
            i128::from_str_radix(&text.replace('_', ""), 10).unwrap_or(0)
        }
    }

    fn parse_duodecimal(&self, text: &str) -> i128 {
        let mut result = 0i128;
        for c in text.chars() {
            result = result * 12 + match c.to_ascii_lowercase() {
                '0'..='9' => c as i128 - '0' as i128,
                'a' | 'b' => c as i128 - 'a' as i128 + 10,
                _ => 0,
            };
        }
        result
    }

    fn parse_tetravigesimal(&self, text: &str) -> i128 {
        let mut result = 0i128;
        for c in text.chars() {
            result = result * 24 + match c.to_ascii_lowercase() {
                '0'..='9' => c as i128 - '0' as i128,
                'a'..='n' => c as i128 - 'a' as i128 + 10,
                _ => 0,
            };
        }
        result
    }

    fn parse_sexagesimal(&self, text: &str) -> i128 {
        let mut result = 0i128;
        for c in text.chars() {
            result = result * 60 + match c.to_ascii_lowercase() {
                '0'..='9' => c as i128 - '0' as i128,
                'a'..='z' => c as i128 - 'a' as i128 + 10,
                _ => 0,
            };
        }
        result
    }

    fn parse_balanced_ternary(&self, text: &str) -> i128 {
        let mut result = 0i128;
        for c in text.chars() {
            result = result * 3 + match c {
                '-' => -1,
                '0' => 0,
                '1' => 1,
                _ => 0,
            };
        }
        result
    }

    fn intern_identifier(&mut self) -> String {
        if let Some(token) = self.tokens.peek() {
            if let Some(span) = self.tokens.position().checked_sub(1) {
                if span < self.tokens.len() {
                    let source = self.get_source_text(span);
                    return source.to_string();
                }
            }
        }
        String::new()
    }

    fn intern_string(&mut self) -> String {
        if let Some(token) = self.tokens.peek() {
            if let Some(span) = self.tokens.position().checked_sub(1) {
                if span < self.tokens.len() {
                    let source = self.get_source_text(span);
                    return source.to_string();
                }
            }
        }
        String::new()
    }

    fn get_source_text(&self, index: usize) -> String {
        String::new()
    }

    fn current_span(&self) -> Result<Span, ChimError> {
        if let Some(token) = self.tokens.peek() {
            Ok(token.span)
        } else {
            Ok(Span::new(self.file_id, 0, 0, 0, 0))
        }
    }

    fn expect(&mut self, expected: Token) -> Result<(), ChimError> {
        if let Some(token) = self.tokens.peek() {
            if token.token == expected {
                self.tokens.next();
                return Ok(());
            }
            
            let error_msg = format!(
                "expected {:?}, found {:?} at {}:{}",
                expected,
                token.token,
                token.span.start_line,
                token.span.start_col
            );
            self.errors.push(ChimError::new(
                ErrorKind::Parser,
                error_msg,
            ).with_span(token.span));
            let err = self.errors.last().cloned().unwrap_or_else(|| {
                ChimError::new(ErrorKind::Parser, "unknown error".to_string())
            });
            Err(err)
        } else {
            let error_msg = format!(
                "expected {:?}, found end of input",
                expected
            );
            self.errors.push(ChimError::new(
                ErrorKind::Parser,
                error_msg,
            ).with_span(Span::new(self.file_id, 0, 0, 0, 0)));
            let err = self.errors.last().cloned().unwrap_or_else(|| {
                ChimError::new(ErrorKind::Parser, "unknown error".to_string())
            });
            Err(err)
        }
    }

    fn expect_one_of(&mut self, expected: &[Token]) -> Result<Token, ChimError> {
        if let Some(token) = self.tokens.peek() {
            if expected.contains(&token.token) {
                let tok = token.token.clone();
                self.tokens.next();
                return Ok(tok);
            }
            
            let expected_str: Vec<String> = expected.iter().map(|t| format!("{:?}", t)).collect();
            let error_msg = format!(
                "expected one of [{}], found {:?} at {}:{}",
                expected_str.join(", "),
                token.token,
                token.span.start_line,
                token.span.start_col
            );
            self.errors.push(ChimError::new(
                ErrorKind::Parser,
                error_msg,
            ).with_span(token.span));
            let err = self.errors.last().cloned().unwrap_or_else(|| {
                ChimError::new(ErrorKind::Parser, "unknown error".to_string())
            });
            Err(err)
        } else {
            let expected_str: Vec<String> = expected.iter().map(|t| format!("{:?}", t)).collect();
            let error_msg = format!(
                "expected one of [{}], found end of input",
                expected_str.join(", ")
            );
            self.errors.push(ChimError::new(
                ErrorKind::Parser,
                error_msg,
            ).with_span(Span::new(self.file_id, 0, 0, 0, 0)));
            let err = self.errors.last().cloned().unwrap_or_else(|| {
                ChimError::new(ErrorKind::Parser, "unknown error".to_string())
            });
            Err(err)
        }
    }

    fn recover(&mut self) {
        self.recover_to_sync_points(&[
            Token::Semicolon,
            Token::LBrace,
            Token::RBrace,
            Token::Func,
            Token::Struct,
            Token::Enum,
            Token::Trait,
            Token::Impl,
            Token::Let,
            Token::LetAlt,
            Token::Var,
        ]);
    }

    fn recover_to_sync_points(&mut self, sync_tokens: &[Token]) {
        while !self.tokens.at_end() {
            if let Some(token) = self.tokens.peek() {
                if sync_tokens.contains(&token.token) {
                    break;
                }
            }
            self.tokens.next();
        }
    }

    fn recover_in_block(&mut self) {
        while !self.tokens.at_end() {
            match self.tokens.peek().map(|t| &t.token) {
                Some(Token::RBrace) => break,
                Some(Token::Func) | Some(Token::Struct) | Some(Token::Enum) | 
                Some(Token::Trait) | Some(Token::Impl) | Some(Token::Let) | 
                Some(Token::LetAlt) | Some(Token::Var) => break,
                _ => { self.tokens.next(); },
            }
        }
    }

    fn recover_in_expr(&mut self) {
        while !self.tokens.at_end() {
            match self.tokens.peek().map(|t| &t.token) {
                Some(Token::Semicolon) | Some(Token::RBrace) | Some(Token::Comma) | 
                Some(Token::RParen) | Some(Token::RBracket) => break,
                _ => { self.tokens.next(); },
            }
        }
    }

    fn skip_to(&mut self, target: Token) -> bool {
        while !self.tokens.at_end() {
            if self.tokens.peek().map(|t| &t.token) == Some(&target) {
                return true;
            }
            self.tokens.next();
        }
        false
    }

    fn report_error(&mut self, kind: ErrorKind, message: String, span: Span) -> ChimError {
        let error = ChimError::new(kind, message).with_span(span);
        self.errors.push(error.clone());
        error
    }

    fn report_error_with_context(&mut self, kind: ErrorKind, message: String, span: Span, context: &str) -> ChimError {
        let full_message = format!("{}: {}", context, message);
        let error = ChimError::new(kind, full_message).with_span(span);
        self.errors.push(error.clone());
        error
    }
}

pub fn parse(source: &str, file_id: FileId) -> Result<Program, Vec<ChimError>> {
    let (tokens, _, _) = chim_lexer::tokenize(source, file_id);
    let mut parser = Parser::new(tokens, &mut lasso::Rodeo::new(), file_id);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_program() {
        let file_id = FileId(0);
        let result = parse("", file_id);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert!(program.items.is_empty());
    }

    #[test]
    fn test_parse_function() {
        let source = r#"
            fn main() -> int {
                return 42;
            }
        "#;
        let file_id = FileId(0);
        let result = parse(source, file_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_struct() {
        let source = r#"
            struct Point {
                x: int;
                y: int;
            }
        "#;
        let file_id = FileId(0);
        let result = parse(source, file_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_enum() {
        let source = r#"
            enum Option {
                Some(int);
                None;
            }
        "#;
        let file_id = FileId(0);
        let result = parse(source, file_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_let_stmt() {
        let source = r#"
            fn main() {
                let x: int = 42;
            }
        "#;
        let file_id = FileId(0);
        let result = parse(source, file_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_match() {
        let source = r#"
            fn main() {
                match x {
                    1 => "one";
                    2 => "two";
                    _ => "other";
                }
            }
        "#;
        let file_id = FileId(0);
        let result = parse(source, file_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_closure() {
        let source = r#"
            fn main() {
                let f = |x: int| x * 2;
            }
        "#;
        let file_id = FileId(0);
        let result = parse(source, file_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_while() {
        let source = r#"
            fn main() {
                while x < 10 {
                    x = x + 1;
                }
            }
        "#;
        let file_id = FileId(0);
        let result = parse(source, file_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_for() {
        let source = r#"
            fn main() {
                for i in 0..10 {
                    println(i);
                }
            }
        "#;
        let file_id = FileId(0);
        let result = parse(source, file_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_array() {
        let source = r#"
            fn main() {
                let arr = [1, 2, 3, 4, 5];
            }
        "#;
        let file_id = FileId(0);
        let result = parse(source, file_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_ternary() {
        let source = r#"
            fn main() {
                let x = a > b ? a : b;
            }
        "#;
        let file_id = FileId(0);
        let result = parse(source, file_id);
        assert!(result.is_ok());
    }
}
