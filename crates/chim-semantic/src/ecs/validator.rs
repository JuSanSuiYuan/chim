use crate::type_pool::TypePool;
use crate::ChimError;
use chim_span::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct EcsComponent {
    pub name: String,
    pub fields: Vec<EcsField>,
    pub storage: StorageType,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EcsField {
    pub name: String,
    pub ty: crate::TypeId,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageType {
    Dense,
    Sparse,
    Table,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EcsSystem {
    pub name: String,
    pub query: EcsQuery,
    pub body: Vec<crate::ast::Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EcsQuery {
    pub read: Vec<String>,
    pub write: Vec<String>,
    pub with: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SystemParam {
    pub name: String,
    pub ty: crate::TypeId,
    pub access: ParamAccess,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamAccess {
    Read,
    Write,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EcsWorld {
    pub components: Vec<EcsComponent>,
    pub systems: Vec<EcsSystem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    pub id: u64,
    pub components: Vec<String>,
    pub span: Span,
}

#[derive(Debug)]
pub struct EcsValidator {
    components: Vec<EcsComponent>,
    systems: Vec<EcsSystem>,
    entities: Vec<Entity>,
    errors: Vec<ChimError>,
}

impl EcsValidator {
    pub fn new() -> Self {
        EcsValidator {
            components: Vec::new(),
            systems: Vec::new(),
            entities: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn validate(&mut self, _pool: &TypePool, _program: &crate::ast::Program) -> Result<(), Vec<ChimError>> {
        Ok(())
    }

    pub fn add_component(&mut self, comp: EcsComponent) {
        self.components.push(comp);
    }

    pub fn add_system(&mut self, sys: EcsSystem) {
        self.systems.push(sys);
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_creation() {
        let pool = TypePool::new();
        let mut validator = EcsValidator::new();

        let comp = EcsComponent {
            name: "Position".to_string(),
            fields: Vec::new(),
            storage: StorageType::Dense,
            span: Span::new(chim_span::FileId(0), 0, 0, 0, 0),
        };

        validator.add_component(comp);
        assert_eq!(validator.components.len(), 1);
    }
}
