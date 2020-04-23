use crate::core::graphic::polygon::polygon_base::PolygonBase;
use crate::math::change_range::ChangeRange;
use crate::math::mesh::Mesh;
use nalgebra_glm::{vec3, vec4, Vec3, Vec4};
use std::rc::Weak;
use crate::core::graphic::polygon::polygon_base_buffer::PolygonBaseBuffer;
use crate::utility::buffer_interface::BufferInterface;

pub struct Polygon {
    pub parent: Option<Weak<dyn PolygonBase>>,
    pub children: Vec<Box<dyn PolygonBase>>,
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
        let position_change_range = ChangeRange::new(mesh.positions.len());
        let color_change_range = ChangeRange::new(mesh.colors.len());
        let texcoord_change_range = ChangeRange::new(mesh.texcoords.len());
        let normal_change_range = ChangeRange::new(mesh.normals.len());
        Polygon {
            parent: None,
            children: vec![],
            visible: true,
            position: vec3(0.0f32, 0.0f32, 0.0f32),
            color: vec4(1.0f32, 1.0f32, 1.0f32, 1.0f32),
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

impl <TBuffer: BufferInterface<f32>> PolygonBaseBuffer<TBuffer> for Polygon {}

#[cfg(test)]
mod tests {
    use crate::core::graphic::polygon::polygon::Polygon;
    use crate::core::graphic::polygon::polygon_base::PolygonBase;
    use crate::core::graphic::polygon::polygon_base_buffer::PolygonBaseBuffer;
    use crate::math::mesh::{
        create_square_colors, create_square_positions, create_square_texcoords, MeshBuilder,
    };
    use crate::utility::buffer_interface::tests::MockBuffer;
    use nalgebra_glm::{vec2, vec3, vec4};
    use std::ops::{Deref, Range};

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

    #[test]
    fn test_tree() {
        let mesh = MeshBuilder::new()
            .with_square(vec2(32.0f32, 64.0f32))
            .build()
            .unwrap();

        // check initial position
        let mut polygon_parent = Polygon::new(mesh.clone());
        polygon_parent.set_color(vec4(0.0f32, 0.0f32, 1.0f32, 1.0f32));

        let mut polygon_child = Polygon::new(mesh.clone());
        polygon_child.set_color(vec4(1.0f32, 0.0f32, 0.0f32, 1.0f32));

        polygon_parent.add_child(Box::new(polygon_child));
        assert_eq!(polygon_parent.children().len(), 1);

        let children = polygon_parent.children();
        let child = children.first().map(|x| x.deref());

        assert_eq!(
            child.map(|x| x.color()),
            Some(&vec4(1.0f32, 0.0f32, 0.0f32, 1.0f32))
        );
    }

    #[test]
    fn test_color() {
        let mesh = MeshBuilder::new()
            .with_square(vec2(32.0f32, 64.0f32))
            .build()
            .unwrap();

        let mut polygon = Polygon::new(mesh.clone());
        assert_eq!(polygon.color(), &vec4(1.0f32, 1.0f32, 1.0f32, 1.0f32));

        polygon.set_color(vec4(1.0f32, 0.0f32, 0.0f32, 1.0f32));
        assert_eq!(polygon.color(), &vec4(1.0f32, 0.0f32, 0.0f32, 1.0f32));

        let mut buffer = MockBuffer::new(256);
        polygon.copy_colors_into(&mut buffer, 0);

        let expected_colors = create_square_colors(vec4(1.0f32, 0.0f32, 0.0f32, 1.0f32))
            .iter()
            .map(|color| vec![color.x, color.y, color.z, color.w])
            .flatten()
            .collect::<Vec<_>>();
        assert_eq!(
            &buffer.data[(buffer.start as usize)..(buffer.end as usize)],
            expected_colors.as_slice()
        );

        let child = Polygon::new(mesh.clone());
        polygon.add_child(Box::new(child));
        polygon.children().iter_mut().for_each(|x| {});
    }

    #[test]
    fn test_texcoord() {
        let mesh = MeshBuilder::new()
            .with_square(vec2(32.0f32, 64.0f32))
            .build()
            .unwrap();

        let mut polygon = Polygon::new(mesh.clone());
        let mut buffer = MockBuffer::new(256);
        let expected_texcoords =
            create_square_texcoords(vec2(0.0f32, 0.0f32), vec2(1.0f32, 1.0f32))
                .iter()
                .map(|texcoord| vec![texcoord.x, texcoord.y])
                .flatten()
                .collect::<Vec<_>>();

        polygon.copy_texcoords_into(&mut buffer, 0);
        assert_eq!(buffer.get_changes(), expected_texcoords.as_slice());
    }

    #[test]
    fn test_normal() {
        let mesh = MeshBuilder::new()
            .with_square(vec2(32.0f32, 64.0f32))
            .build()
            .unwrap();

        let mut polygon = Polygon::new(mesh.clone());
        let mut buffer = MockBuffer::new(256);
        polygon.copy_normals_into(&mut buffer, 0);

        // Allow empty for 2d
        assert_eq!(buffer.get_changes(), []);
    }
}
