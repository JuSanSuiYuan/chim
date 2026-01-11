pub mod type_pool;
pub mod type_inference;
pub mod type_inference;
pub mod memory_safety;
pub mod lifetime;
pub mod borrow_check;
pub mod ecs;
pub mod actor;
pub mod math_verification;

pub use type_pool::{TypePool, TypeId, StructId, EnumId, TraitId, FunctionId, VarId, ConstId, LifetimeId, TypeData, StructData, EnumData, TraitData, FunctionData, FunctionSig, BuiltinTypes, IntSize, UintSize, FloatSize, Mutability, TagRepresentation, LifetimeData, LifetimeKind};
pub use type_inference::{TypeInferencer, TypeConstraints};
pub use type_inference::enhanced_inferencer::{EnhancedTypeInferencer, TypeVar, TypeKind, Kind, Substitution, InferenceConfig};
pub use memory_safety::{BoundaryChecker, BoundaryCheck, BoundaryCheckType, CastChecker, CastCheck, LinearTypeChecker, LinearType, UsageInfo, NullSafetyChecker, NullableType};
pub use lifetime::{LifetimeAnalyzer, LifetimeResult, LifetimeConstraint, AllocationLifetime};
pub use borrow_check::{BorrowChecker, Borrow, BorrowKind, BorrowTarget, Variable, VariableId, BorrowId};
pub use ecs::{EcsValidator, EcsComponent, EcsSystem, EcsWorld, Entity, StorageType, EcsQuery, SystemParam, ParamAccess};
pub use actor::{ActorValidator, Actor, ActorState, MessageHandler, Message, ActorRef, ActorRuntime, Channel, MailboxConfig, Priority, Ordering};
pub use math_verification::{DependentTypeChecker, DependentType, TypeConstraint, DependentTypeError, DependentTypeErrorKind};
pub use math_verification::{LinearTypeChecker, LinearType, LinearTypeError, LinearTypeErrorKind};
pub use math_verification::{EffectTypeChecker, EffectType, Effect, EffectTypeError, EffectTypeErrorKind};
pub use math_verification::{SessionTypeChecker, SessionType, SessionTypeError, SessionTypeErrorKind};
pub use math_verification::{ProofGenerator, Proof, ProofGenerationError, ProofGenerationErrorKind};

use chim_span::{Span, FileId};
use chim_error::{ChimError, ErrorKind};
use chim_ast::Program;
use std::sync::Arc;

pub struct SemanticAnalyzer {
    pool: TypePool,
    type_inferencer: TypeInferencer,
    lifetime_analyzer: LifetimeAnalyzer,
    borrow_checker: BorrowChecker,
    ecs_validator: EcsValidator,
    actor_validator: ActorValidator,
    dependent_type_checker: DependentTypeChecker,
    linear_type_checker: LinearTypeChecker,
    effect_type_checker: EffectTypeChecker,
    session_type_checker: SessionTypeChecker,
    proof_generator: ProofGenerator,
    errors: Vec<ChimError>,
    warnings: Vec<ChimError>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let pool = TypePool::new();
        let type_inferencer = TypeInferencer::new();
        let lifetime_analyzer = LifetimeAnalyzer::new();
        let borrow_checker = BorrowChecker::new();
        let ecs_validator = EcsValidator::new();
        let actor_validator = ActorValidator::new();
        let dependent_type_checker = DependentTypeChecker::new();
        let linear_type_checker = LinearTypeChecker::new();
        let effect_type_checker = EffectTypeChecker::new();
        let session_type_checker = SessionTypeChecker::new();
        let proof_generator = ProofGenerator::new();

        SemanticAnalyzer {
            pool,
            type_inferencer,
            lifetime_analyzer,
            borrow_checker,
            ecs_validator,
            actor_validator,
            dependent_type_checker,
            linear_type_checker,
            effect_type_checker,
            session_type_checker,
            proof_generator,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<AnalyzedProgram, Vec<ChimError>> {
        self.errors.clear();
        self.warnings.clear();

        let mut type_inferencer = TypeInferencer::new();
        
        if let Err(type_errors) = type_inferencer.infer_program(program) {
            self.errors.extend(type_errors);
            return Err(std::mem::take(&mut self.errors));
        }

        self.pool = type_inferencer.take_pool();

        let mut lifetime_analyzer = LifetimeAnalyzer::new();
        let lifetime_result = match lifetime_analyzer.analyze_program(program, &self.pool) {
            Ok(result) => result,
            Err(errors) => {
                self.errors.extend(errors);
                LifetimeResult::new()
            }
        };

        let mut borrow_checker = BorrowChecker::new();
        let _ = borrow_checker.check_program(program, &self.pool, &lifetime_result);

        let mut dependent_type_checker = DependentTypeChecker::new();
        for item in &program.items {
            if let Err(errors) = dependent_type_checker.check_item(item) {
                self.errors.extend(errors);
            }
        }

        let mut linear_type_checker = LinearTypeChecker::new();
        for item in &program.items {
            if let Err(errors) = linear_type_checker.check_item(item) {
                self.errors.extend(errors);
            }
        }

        let mut effect_type_checker = EffectTypeChecker::new();
        for item in &program.items {
            if let Err(errors) = effect_type_checker.check_item(item) {
                self.errors.extend(errors);
            }
        }

        let mut session_type_checker = SessionTypeChecker::new();
        for item in &program.items {
            if let Err(errors) = session_type_checker.check_item(item) {
                self.errors.extend(errors);
            }
        }

        let mut proof_generator = ProofGenerator::new();
        for item in &program.items {
            if let Err(errors) = proof_generator.generate_item_proofs(item) {
                self.errors.extend(errors);
            }
        }

        let ecs_world = None;
        let actor_runtime = None;

        if self.errors.is_empty() {
            Ok(AnalyzedProgram {
                pool: std::mem::take(&mut self.pool),
                lifetime_result,
                ecs_world,
                actor_runtime,
            })
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    pub fn take_errors(&mut self) -> Vec<ChimError> {
        std::mem::take(&mut self.errors)
    }

    pub fn take_warnings(&mut self) -> Vec<ChimError> {
        std::mem::take(&mut self.warnings)
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    pub fn pool(&self) -> &TypePool {
        &self.pool
    }

    pub fn pool_mut(&mut self) -> &mut TypePool {
        &mut self.pool
    }
}

#[derive(Debug, Clone)]
pub struct AnalyzedProgram {
    pub pool: TypePool,
    pub lifetime_result: LifetimeResult,
    pub ecs_world: Option<EcsWorld>,
    pub actor_runtime: Option<ActorRuntime>,
}

impl AnalyzedProgram {
    pub fn type_of(&self, ty_id: TypeId) -> &TypeData {
        self.pool.get_type(ty_id)
    }

    pub fn struct_of(&self, struct_id: StructId) -> &StructData {
        self.pool.get_struct(struct_id)
    }

    pub fn enum_of(&self, enum_id: EnumId) -> &EnumData {
        self.pool.get_enum(enum_id)
    }

    pub fn type_size(&self, ty_id: TypeId) -> usize {
        self.pool.type_size(ty_id)
    }

    pub fn type_align(&self, ty_id: TypeId) -> usize {
        self.pool.type_align(ty_id)
    }

    pub fn is_stack_allocated(&self, ty_id: TypeId) -> bool {
        self.pool.is_stack_allocated(ty_id)
    }

    pub fn is_heap_allocated(&self, ty_id: TypeId) -> bool {
        self.pool.is_heap_allocated(ty_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = SemanticAnalyzer::new();
        assert_eq!(analyzer.error_count(), 0);
        assert_eq!(analyzer.warning_count(), 0);
    }

    #[test]
    fn test_analyzed_program_type_lookup() {
        let mut analyzer = SemanticAnalyzer::new();
        let program = Program {
            items: Vec::new(),
            span: Span::new(FileId(0), 0, 0, 0, 0),
        };

        let result = analyzer.analyze(&program);
        assert!(result.is_ok());
    }
}
