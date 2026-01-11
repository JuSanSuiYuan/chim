pub mod generator;

pub use generator::{
    AgdaFFIGenerator, AgdaType, AgdaFunction, AgdaParameter, AgdaData,
    AgdaConstructor, AgdaField, AgdaRecord, AgdaModule, AgdaDeclaration,
    map_chim_type_to_agda,
};
