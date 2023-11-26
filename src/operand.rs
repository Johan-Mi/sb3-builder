use crate::{Input, VariableRef};

pub struct Operand(pub(crate) Input);

impl From<f64> for Operand {
    fn from(value: f64) -> Self {
        Self(Input::Number(value))
    }
}

impl From<VariableRef> for Operand {
    fn from(value: VariableRef) -> Self {
        Self(Input::Variable(value))
    }
}
