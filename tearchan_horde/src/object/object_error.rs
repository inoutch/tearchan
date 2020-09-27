#[derive(Debug)]
pub enum ObjectError {
    FactoryNotRegistered,
    CreationFailed,
    InvalidType,
}
