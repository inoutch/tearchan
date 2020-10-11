use downcast_rs::Downcast;

pub trait GameObjectBase: Downcast {}
impl_downcast!(GameObjectBase);
