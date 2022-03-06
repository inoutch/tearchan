use bytemuck::Pod;

pub trait Primitive: Pod {}

impl Primitive for i32 {}
impl Primitive for f32 {}
impl Primitive for u32 {}
impl Primitive for f64 {}
impl Primitive for u64 {}
impl Primitive for usize {}
