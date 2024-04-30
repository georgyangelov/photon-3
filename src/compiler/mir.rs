use crate::compiler::lexical_scope::LocalSlotRef;
use crate::frontend::Location;

pub struct MIR {
    pub node: MIRNode,
    pub typ: MIRType,
    pub location: Location
}

pub enum MIRNode {
    // GlobalRef(i32),

    LocalSet(LocalSlotRef, Box<MIR>),
    LocalRef(LocalSlotRef),

    Block(Vec<MIR>),

    Call(FunctionRef, Vec<MIR>)
}

pub enum MIRType {

}

#[derive(Copy)]
pub struct FunctionRef {
    i: i32
}