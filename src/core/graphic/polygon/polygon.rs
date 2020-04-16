use crate::core::graphic::polygon::polygon_base::PolygonBase;
use crate::math::change_range::ChangeRange;
use crate::math::mesh::Mesh;
use nalgebra_glm::{vec3, vec4, Vec3, Vec4};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct Polygon {
    pub parent: Option<Weak<dyn PolygonBase>>,
    pub children: Vec<Rc<RefCell<dyn PolygonBase>>>,
    pub visible: bool,
    pub position: Vec3,
    pub color: Vec4,
    pub scale: Vec3,
    pub mesh: Mesh,
    pub position_change_range: ChangeRange,
    pub color_change_range: ChangeRange,
    pub texcoord_change_range: ChangeRange,
    pub normal_change_range: ChangeRange,
}

impl Polygon {
    pub fn new(mesh: Mesh) -> Polygon {
        let position_change_range = ChangeRange::new(mesh.positions.len() as u32);
        let color_change_range = ChangeRange::new(mesh.colors.len() as u32);
        let texcoord_change_range = ChangeRange::new(mesh.texcoords.len() as u32);
        let normal_change_range = ChangeRange::new(mesh.normals.len() as u32);
        Polygon {
            parent: None,
            children: vec![],
            visible: true,
            position: vec3(0.0f32, 0.0f32, 0.0f32),
            color: vec4(0.0f32, 0.0f32, 0.0f32, 1.0f32),
            scale: vec3(1.0f32, 1.0f32, 1.0f32),
            mesh,
            position_change_range,
            color_change_range,
            texcoord_change_range,
            normal_change_range,
        }
    }
}

impl PolygonBase for Polygon {
    fn get_mut_polygon(&mut self) -> &mut Polygon {
        self
    }

    fn get_polygon(&self) -> &Polygon {
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::core::graphic::polygon::polygon::Polygon;
    use crate::core::graphic::polygon::polygon_base::PolygonBase;
    use crate::core::graphic::polygon::polygon_base_buffer::PolygonBaseBuffer;
    use crate::math::mesh::{create_square_positions, MeshBuilder};
    use crate::utility::buffer_interface::tests::MockBuffer;
    use nalgebra_glm::{vec2, vec3};
    use std::ops::Range;

    impl PolygonBaseBuffer<MockBuffer> for Polygon {}

    #[test]
    fn test_position() {
        let mesh = MeshBuilder::new()
            .with_square(vec2(32.0f32, 64.0f32))
            .build()
            .unwrap();

        // check initial position
        let mut polygon = Polygon::new(mesh);
        assert_eq!(polygon.position(), &vec3(0.0f32, 0.0f32, 0.0f32));

        // check updated position
        polygon.set_position(vec3(1.0f32, 2.0f32, 3.0f32));
        assert_eq!(polygon.position(), &vec3(1.0f32, 2.0f32, 3.0f32));

        // check change range
        let range = polygon.position_change_range().get_range();
        assert_eq!(range, Some(Range { start: 0, end: 6 }));

        // copy position
        let mut mock_buffer = MockBuffer::new(256);
        polygon.copy_positions_into(&mut mock_buffer, 0);

        let slice = &mock_buffer.data[(mock_buffer.start as usize)..(mock_buffer.end as usize)];
        let vertices = create_square_positions(vec2(0.0f32, 0.0f32), vec2(32.0f32, 64.0f32))
            .iter()
            .map(|x| x + vec3(1.0f32, 2.0f32, 3.0f32))
            .map(|x| vec![x.x, x.y, x.z])
            .flatten()
            .collect::<Vec<_>>();
        // check copied buffer
        assert_eq!(slice, vertices.as_slice());

        let mut range = polygon.position_change_range().get_range();
        // check reset change range
        assert_eq!(range, None);

        polygon.set_visible(false);
        range = polygon.position_change_range().get_range();
        assert_eq!(range, Some(Range { start: 0, end: 6 }));
    }
}
