use crate::Input;

pub struct Operand(pub(crate) Input);

impl From<f64> for Operand {
    fn from(value: f64) -> Self {
        Self(Input::Number(value))
    }
}
