use gfx_hal::pso::AttributeDesc;

#[derive(Debug)]
pub struct Attribute {
    pub attribute_desc: AttributeDesc,
    pub stride: u32,
}
