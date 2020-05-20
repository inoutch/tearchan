use crate::extension::shared::{Shared, SharedWeak};
use crate::math::change_range::ChangeRange;
use crate::math::mesh::Mesh;
use crate::math::vec::{make_vec3_fill, make_vec3_zero, make_vec4_white};
use crate::utility::buffer_interface::BufferInterface;
use nalgebra_glm::{rotate, scale, translate, vec3, vec3_to_vec4, vec4, Mat4, Vec3, Vec4};
use std::cell::RefCell;
use std::option::Option::Some;
use std::rc::Rc;

pub trait PolygonProvider {
    fn position<'a>(&self, core: &'a PolygonCore) -> &'a Vec3;
    fn set_position(&mut self, core: &mut PolygonCore, position: Vec3);
    fn color<'a>(&self, core: &'a PolygonCore) -> &'a Vec4;
    fn set_color(&mut self, core: &mut PolygonCore, color: Vec4);
    fn computed_color(&self, core: &PolygonCore) -> Vec4;
    fn scale<'a>(&self, core: &'a PolygonCore) -> &'a Vec3;
    fn set_scale(&mut self, core: &mut PolygonCore, scale: Vec3);
    fn rotation_axis<'a>(&self, core: &'a PolygonCore) -> &'a Vec3;
    fn set_rotation_axis(&mut self, core: &mut PolygonCore, rotation_axis: Vec3);
    fn rotation_radian(&self, core: &PolygonCore) -> f32;
    fn set_rotation_radian(&mut self, core: &mut PolygonCore, rotation_radian: f32);
    fn visible(&self, core: &PolygonCore) -> bool;
    fn set_visible(&mut self, core: &mut PolygonCore, visible: bool);
    fn computed_visible(&self, core: &PolygonCore) -> bool;
    fn add_child(&mut self, core: &mut PolygonCore, polygon: &Shared<Polygon>);
    fn transform(&self, core: &PolygonCore) -> Mat4;
    fn transform_for_child(&self, core: &PolygonCore) -> Mat4;
}

pub trait PolygonCommon {
    fn mesh(&self) -> &Mesh;
    fn position(&self) -> &Vec3;
    fn set_position(&mut self, position: Vec3);
    fn color(&self) -> &Vec4;
    fn set_color(&mut self, color: Vec4);
    fn computed_color(&self) -> Vec4;
    fn scale(&self) -> &Vec3;
    fn set_scale(&mut self, scale: Vec3);
    fn rotation_axis(&self) -> &Vec3;
    fn set_rotation_axis(&mut self, rotation_axis: Vec3);
    fn rotation_radian(&self) -> f32;
    fn set_rotation_radian(&mut self, rotation_radian: f32);
    fn visible(&self) -> bool;
    fn set_visible(&mut self, visible: bool);
    fn computed_visible(&self) -> bool;
    fn add_child(&mut self, polygon: &Shared<Polygon>);
}

pub struct Polygon {
    core: PolygonCore,
    provider: Box<dyn PolygonProvider>,
}

impl Polygon {
    pub fn new(mesh: Mesh) -> Polygon {
        Polygon::new_with_provider(Box::new(PolygonDefaultProvider {}), mesh)
    }

    pub fn new_with_provider(provider: Box<dyn PolygonProvider>, mesh: Mesh) -> Polygon {
        let position_change_range = ChangeRange::new(mesh.positions.len());
        let color_change_range = ChangeRange::new(mesh.colors.len());
        let texcoord_change_range = ChangeRange::new(mesh.texcoords.len());
        let normal_change_range = ChangeRange::new(mesh.normals.len());
        Polygon {
            core: PolygonCore {
                mesh,
                position: make_vec3_zero(),
                color: make_vec4_white(),
                scale: make_vec3_fill(1.0f32),
                visible: true,
                rotation_axis: vec3(0.0f32, 0.0f32, 1.0f32),
                rotation_radian: 0.0f32,
                parent: None,
                children: vec![],
                position_change_range,
                color_change_range,
                texcoord_change_range,
                normal_change_range,
            },
            provider,
        }
    }

    #[inline]
    pub fn transform_for_child(&self) -> Mat4 {
        self.provider.transform_for_child(&self.core)
    }

    pub fn copy_positions_into<TBuffer: BufferInterface<f32>>(
        &mut self,
        buffer: &mut TBuffer,
        offset: usize,
    ) {
        let change_range = &self.core.position_change_range;
        if let Some(range) = change_range.get_range() {
            let matrix = self.provider.transform(&self.core);
            let mesh_positions = &self.mesh().positions;
            let position = self.position();

            buffer.update_with_range(range.start * 3, range.end * 3);
            if self.computed_visible() {
                for i in range {
                    let mesh_position = &mesh_positions[i as usize];
                    let m = &matrix;
                    let v = m * vec3_to_vec4(&(position + mesh_position));
                    buffer.copy(offset + i * 3, v.x);
                    buffer.copy(offset + i * 3 + 1, v.y);
                    buffer.copy(offset + i * 3 + 2, v.z);
                }
            } else {
                for i in range {
                    buffer.copy(offset + i * 3, 0.0f32);
                    buffer.copy(offset + i * 3 + 1, 0.0f32);
                    buffer.copy(offset + i * 3 + 2, 0.0f32);
                }
            }
            self.core.reset_all_position_change_range();
        }
    }

    pub fn copy_colors_into<TBuffer: BufferInterface<f32>>(
        &mut self,
        buffer: &mut TBuffer,
        offset: usize,
    ) {
        let change_range = &self.core.color_change_range;
        if let Some(range) = change_range.get_range() {
            let color = if let Some(x) = self.core.parent() {
                x.borrow().computed_color()
            } else {
                self.computed_color()
            };
            let mesh_colors = &self.mesh().colors;

            buffer.update_with_range(range.start * 4, range.end * 4);
            for i in range {
                let base_color = &mesh_colors[i as usize];
                buffer.copy(offset + i * 4, color.x * base_color.x);
                buffer.copy(offset + i * 4 + 1, color.y * base_color.y);
                buffer.copy(offset + i * 4 + 2, color.z * base_color.z);
                buffer.copy(offset + i * 4 + 3, color.w * base_color.w);
            }
            self.core.reset_all_color_change_range();
        }
    }

    pub fn copy_texcoords_into<TBuffer: BufferInterface<f32>>(
        &mut self,
        buffer: &mut TBuffer,
        offset: usize,
    ) {
        let change_range = &self.core.texcoord_change_range;
        if let Some(range) = change_range.get_range() {
            let mesh_texcoords = &self.mesh().texcoords;

            buffer.update_with_range(range.start * 2, range.end * 2);
            for i in range {
                let uv = &mesh_texcoords[i as usize];
                buffer.copy(offset + i * 2, uv.x);
                buffer.copy(offset + i * 2 + 1, uv.y);
            }
            self.core.reset_all_texcoord_change_range();
        }
    }

    pub fn copy_normals_into<TBuffer: BufferInterface<f32>>(
        &mut self,
        buffer: &mut TBuffer,
        offset: usize,
    ) {
        let change_range = &self.core.normal_change_range;
        if let Some(range) = change_range.get_range() {
            let matrix = self.provider.transform(&self.core);
            let mesh_normals = &self.mesh().normals;

            buffer.update_with_range(range.start * 3, range.end * 3);
            for i in range {
                let m = &matrix;
                let v = m * vec3_to_vec4(&mesh_normals[i as usize]);
                buffer.copy(offset + i * 3, v.x);
                buffer.copy(offset + i * 3 + 1, v.y);
                buffer.copy(offset + i * 3 + 2, v.z);
            }
            self.core.reset_all_normal_change_range();
        }
    }

    pub fn mesh_size(&self) -> usize {
        self.mesh().size()
    }
}

impl PolygonCommon for Polygon {
    #[inline]
    fn mesh(&self) -> &Mesh {
        self.core.mesh()
    }

    #[inline]
    fn position(&self) -> &Vec3 {
        self.provider.position(&self.core)
    }

    #[inline]
    fn set_position(&mut self, position: Vec3) {
        self.provider.set_position(&mut self.core, position);
    }

    #[inline]
    fn color(&self) -> &Vec4 {
        self.provider.color(&self.core)
    }

    #[inline]
    fn set_color(&mut self, color: Vec4) {
        self.provider.set_color(&mut self.core, color)
    }

    #[inline]
    fn computed_color(&self) -> Vec4 {
        self.provider.computed_color(&self.core)
    }

    #[inline]
    fn scale(&self) -> &Vec3 {
        self.provider.scale(&self.core)
    }

    #[inline]
    fn set_scale(&mut self, scale: Vec3) {
        self.provider.set_scale(&mut self.core, scale);
    }

    #[inline]
    fn rotation_axis(&self) -> &Vec3 {
        self.provider.rotation_axis(&self.core)
    }

    #[inline]
    fn set_rotation_axis(&mut self, rotation_axis: Vec3) {
        self.provider
            .set_rotation_axis(&mut self.core, rotation_axis);
    }

    #[inline]
    fn rotation_radian(&self) -> f32 {
        self.provider.rotation_radian(&self.core)
    }

    #[inline]
    fn set_rotation_radian(&mut self, rotation_radian: f32) {
        self.provider
            .set_rotation_radian(&mut self.core, rotation_radian);
    }

    #[inline]
    fn visible(&self) -> bool {
        self.provider.visible(&self.core)
    }

    #[inline]
    fn set_visible(&mut self, visible: bool) {
        self.provider.set_visible(&mut self.core, visible);
    }

    #[inline]
    fn computed_visible(&self) -> bool {
        self.provider.computed_visible(&self.core)
    }

    #[inline]
    fn add_child(&mut self, polygon: &Shared<Polygon>) {
        self.provider.add_child(&mut self.core, polygon);
    }
}

pub struct PolygonCore {
    mesh: Mesh,
    position: Vec3,
    color: Vec4,
    scale: Vec3,
    rotation_axis: Vec3,
    rotation_radian: f32,
    visible: bool,
    parent: Option<SharedWeak<Polygon>>,
    children: Vec<Shared<Polygon>>,
    position_change_range: ChangeRange,
    color_change_range: ChangeRange,
    texcoord_change_range: ChangeRange,
    normal_change_range: ChangeRange,
}

impl PolygonCore {
    #[inline]
    pub fn transform(&self, provider: &dyn PolygonProvider) -> Mat4 {
        let current = scale(
            &rotate(
                &translate(&Mat4::identity(), provider.position(self)),
                provider.rotation_radian(self),
                provider.rotation_axis(self),
            ),
            provider.scale(self),
        );
        if let Some(x) = self.parent() {
            return x.borrow().transform_for_child() * current;
        }
        current
    }

    #[inline]
    pub fn transform_for_children(&self, provider: &dyn PolygonProvider) -> Mat4 {
        provider.transform(self)
    }

    pub fn update_all_positions(&mut self) {
        self.position_change_range.update_all();
        self.children.iter_mut().for_each(|x| {
            x.borrow_mut().core.update_all_positions();
        });
    }

    pub fn update_all_colors(&mut self) {
        self.color_change_range.update_all();
        self.children.iter_mut().for_each(|x| {
            x.borrow_mut().core.update_all_colors();
        });
    }

    pub fn update_all_texcoords(&mut self) {
        self.texcoord_change_range.update_all();
        self.children.iter_mut().for_each(|x| {
            x.borrow_mut().core.update_all_texcoords();
        });
    }

    pub fn update_all_normals(&mut self) {
        self.normal_change_range.update_all();
        self.children.iter_mut().for_each(|x| {
            x.borrow_mut().core.update_all_normals();
        });
    }

    pub fn reset_all_position_change_range(&mut self) {
        self.position_change_range.reset();
    }

    pub fn reset_all_color_change_range(&mut self) {
        self.color_change_range.reset();
    }

    pub fn reset_all_texcoord_change_range(&mut self) {
        self.texcoord_change_range.reset();
    }

    pub fn reset_all_normal_change_range(&mut self) {
        self.normal_change_range.reset();
    }

    pub fn parent(&self) -> Option<Rc<RefCell<Polygon>>> {
        if let Some(x) = &self.parent {
            if let Some(y) = x.upgrade() {
                return Some(y);
            }
        }
        None
    }
}

impl PolygonCommon for PolygonCore {
    #[inline]
    fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    #[inline]
    fn position(&self) -> &Vec3 {
        &self.position
    }

    #[inline]
    fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.update_all_positions();
    }

    #[inline]
    fn color(&self) -> &Vec4 {
        &self.color
    }

    #[inline]
    fn set_color(&mut self, color: Vec4) {
        self.color = color;
        self.color_change_range.update_all();
    }

    #[inline]
    fn computed_color(&self) -> Vec4 {
        if let Some(x) = self.parent() {
            let parent_color = x.borrow().computed_color();
            return vec4(
                parent_color.x * self.color.x,
                parent_color.y * self.color.y,
                parent_color.z * self.color.z,
                parent_color.w * self.color.w,
            );
        }
        self.color.clone_owned()
    }

    #[inline]
    fn scale(&self) -> &Vec3 {
        &self.scale
    }

    #[inline]
    fn set_scale(&mut self, scale: Vec3) {
        if scale != self.scale {
            self.scale = scale;
            self.update_all_positions();
        }
    }

    #[inline]
    fn rotation_axis(&self) -> &Vec3 {
        &self.rotation_axis
    }

    #[inline]
    fn set_rotation_axis(&mut self, rotation_axis: Vec3) {
        if rotation_axis != self.rotation_axis {
            self.rotation_axis = rotation_axis;
            self.update_all_positions();
            self.update_all_normals();
        }
    }

    #[inline]
    fn rotation_radian(&self) -> f32 {
        self.rotation_radian
    }

    #[inline]
    fn set_rotation_radian(&mut self, rotation_radian: f32) {
        self.rotation_radian = rotation_radian;
        self.update_all_positions();
        self.update_all_normals();
    }

    #[inline]
    fn visible(&self) -> bool {
        self.visible
    }

    #[inline]
    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
        self.update_all_positions();
    }

    #[inline]
    fn computed_visible(&self) -> bool {
        if let Some(x) = self.parent() {
            return x.borrow().computed_visible();
        }
        self.visible()
    }

    #[inline]
    fn add_child(&mut self, polygon: &Shared<Polygon>) {
        self.children.push(Shared::clone(polygon))
    }
}

pub struct PolygonDefaultProvider {}

impl PolygonProvider for PolygonDefaultProvider {
    fn position<'a>(&self, core: &'a PolygonCore) -> &'a Vec3 {
        core.position()
    }

    fn set_position(&mut self, core: &mut PolygonCore, position: Vec3) {
        core.set_position(position);
    }

    fn color<'a>(&self, core: &'a PolygonCore) -> &'a Vec4 {
        core.color()
    }

    fn set_color(&mut self, core: &mut PolygonCore, color: Vec4) {
        core.set_color(color);
    }

    fn computed_color(&self, core: &PolygonCore) -> Vec4 {
        core.computed_color()
    }

    fn scale<'a>(&self, core: &'a PolygonCore) -> &'a Vec3 {
        core.scale()
    }

    fn set_scale(&mut self, core: &mut PolygonCore, scale: Vec3) {
        core.set_scale(scale);
    }

    fn rotation_axis<'a>(&self, core: &'a PolygonCore) -> &'a Vec3 {
        core.rotation_axis()
    }

    fn set_rotation_axis(&mut self, core: &mut PolygonCore, rotation_axis: Vec3) {
        core.set_rotation_axis(rotation_axis);
    }

    fn rotation_radian(&self, core: &PolygonCore) -> f32 {
        core.rotation_radian()
    }

    fn set_rotation_radian(&mut self, core: &mut PolygonCore, rotation_radian: f32) {
        core.set_rotation_radian(rotation_radian);
    }

    fn visible(&self, core: &PolygonCore) -> bool {
        core.visible()
    }

    fn set_visible(&mut self, core: &mut PolygonCore, visible: bool) {
        core.set_visible(visible);
    }

    fn computed_visible(&self, core: &PolygonCore) -> bool {
        core.computed_visible()
    }

    fn add_child(&mut self, core: &mut PolygonCore, polygon: &Shared<Polygon>) {
        core.add_child(polygon);
    }

    fn transform(&self, core: &PolygonCore) -> Mat4 {
        core.transform(self)
    }

    fn transform_for_child(&self, core: &PolygonCore) -> Mat4 {
        core.transform_for_children(self)
    }
}

#[cfg(test)]
mod test {
    use crate::core::graphic::polygon::{Polygon, PolygonCommon, PolygonCore, PolygonProvider};
    use crate::extension::shared::Shared;
    use crate::math::mesh::{
        create_square_colors, create_square_positions, create_square_texcoords, MeshBuilder,
    };
    use crate::utility::buffer_interface::tests::MockBuffer;
    use crate::utility::test::func::MockFunc;
    use nalgebra_glm::{vec2, vec3, vec4, Mat4, Vec3, Vec4};
    use std::ops::Range;

    struct MockPolygonProvider {
        mock: Shared<MockFunc>,
    }

    impl PolygonProvider for MockPolygonProvider {
        fn position<'a>(&self, core: &'a PolygonCore) -> &'a Vec3 {
            self.mock.borrow_mut().call(vec!["position".to_string()]);
            core.position()
        }

        fn set_position(&mut self, core: &mut PolygonCore, position: Vec3) {
            self.mock
                .borrow_mut()
                .call(vec!["set_position".to_string()]);
            core.set_position(position);
        }

        fn color<'a>(&self, core: &'a PolygonCore) -> &'a Vec4 {
            self.mock.borrow_mut().call(vec!["color".to_string()]);
            core.color()
        }

        fn set_color(&mut self, core: &mut PolygonCore, color: Vec4) {
            self.mock.borrow_mut().call(vec!["set_color".to_string()]);
            core.set_color(color);
        }

        fn computed_color(&self, core: &PolygonCore) -> Vec4 {
            self.mock
                .borrow_mut()
                .call(vec!["computed_color".to_string()]);
            core.computed_color()
        }

        fn scale<'a>(&self, core: &'a PolygonCore) -> &'a Vec3 {
            self.mock.borrow_mut().call(vec!["scale".to_string()]);
            core.scale()
        }

        fn set_scale(&mut self, core: &mut PolygonCore, scale: Vec3) {
            self.mock.borrow_mut().call(vec!["set_scale".to_string()]);
            core.set_scale(scale);
        }

        fn rotation_axis<'a>(&self, core: &'a PolygonCore) -> &'a Vec3 {
            self.mock.borrow_mut().call(vec!["color".to_string()]);
            core.rotation_axis()
        }

        fn set_rotation_axis(&mut self, core: &mut PolygonCore, rotation_axis: Vec3) {
            self.mock
                .borrow_mut()
                .call(vec!["set_rotation_axis".to_string()]);
            core.set_rotation_axis(rotation_axis);
        }

        fn rotation_radian<'a>(&self, core: &PolygonCore) -> f32 {
            self.mock
                .borrow_mut()
                .call(vec!["rotation_radian".to_string()]);
            core.rotation_radian()
        }

        fn set_rotation_radian(&mut self, core: &mut PolygonCore, rotation_radian: f32) {
            self.mock
                .borrow_mut()
                .call(vec!["set_rotation_radian".to_string()]);
            core.set_rotation_radian(rotation_radian);
        }

        fn visible(&self, core: &PolygonCore) -> bool {
            self.mock.borrow_mut().call(vec!["visible".to_string()]);
            core.visible()
        }

        fn set_visible(&mut self, core: &mut PolygonCore, visible: bool) {
            self.mock.borrow_mut().call(vec!["set_visible".to_string()]);
            core.set_visible(visible);
        }

        fn computed_visible(&self, core: &PolygonCore) -> bool {
            self.mock
                .borrow_mut()
                .call(vec!["computed_visible".to_string()]);
            core.computed_visible()
        }

        fn add_child(&mut self, core: &mut PolygonCore, polygon: &Shared<Polygon>) {
            self.mock.borrow_mut().call(vec!["add_child".to_string()]);
            core.add_child(polygon);
        }

        fn transform(&self, core: &PolygonCore) -> Mat4 {
            self.mock.borrow_mut().call(vec!["transform".to_string()]);
            core.transform(self)
        }

        fn transform_for_child(&self, core: &PolygonCore) -> Mat4 {
            self.mock
                .borrow_mut()
                .call(vec!["transform_for_child".to_string()]);
            core.transform_for_children(self)
        }
    }

    #[test]
    fn test_init() {
        let mock = Shared::new(MockFunc::new());
        let provider = MockPolygonProvider {
            mock: Shared::clone(&mock),
        };
        let mesh = MeshBuilder::new()
            .with_square(vec2(32.0f32, 64.0f32))
            .build()
            .unwrap();

        let polygon = Polygon::new_with_provider(Box::new(provider), mesh);
        assert_eq!(polygon.position(), &vec3(0.0f32, 0.0f32, 0.0f32));
        assert_eq!(polygon.color(), &vec4(1.0f32, 1.0f32, 1.0f32, 1.0f32));
        assert_eq!(polygon.scale(), &vec3(1.0f32, 1.0f32, 1.0f32));
        assert_eq!(polygon.rotation_axis(), &vec3(0.0f32, 0.0f32, 1.0f32));
        assert!(float_cmp::approx_eq!(
            f32,
            polygon.rotation_radian(),
            0.0f32,
            ulps = 2
        ));
        assert_eq!(polygon.visible(), true);
    }

    #[test]
    fn test_transform() {
        let mock = Shared::new(MockFunc::new());
        let mesh = MeshBuilder::new()
            .with_square(vec2(32.0f32, 64.0f32))
            .build()
            .unwrap();
        let polygon = Polygon::new_with_provider(
            Box::new(MockPolygonProvider {
                mock: Shared::clone(&mock),
            }),
            mesh,
        );
        assert_eq!(polygon.provider.transform(&polygon.core), Mat4::identity());
    }

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
        let range = polygon.core.position_change_range.get_range();
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

        let mut range = polygon.core.position_change_range.get_range();
        // check reset change range
        assert_eq!(range, None);

        polygon.set_visible(false);
        range = polygon.core.position_change_range.get_range();
        assert_eq!(range, Some(Range { start: 0, end: 6 }));
    }

    #[test]
    fn test_color() {
        let mesh = MeshBuilder::new()
            .with_square(vec2(32.0f32, 64.0f32))
            .build()
            .unwrap();

        let mut polygon = Polygon::new(mesh);
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
    }

    #[test]
    fn test_texcoord() {
        let mesh = MeshBuilder::new()
            .with_square(vec2(32.0f32, 64.0f32))
            .build()
            .unwrap();

        let mut polygon = Polygon::new(mesh);
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

    #[test]
    fn test_tree() {
        let mock = Shared::new(MockFunc::new());
        let mesh = MeshBuilder::new()
            .with_square(vec2(32.0f32, 64.0f32))
            .build()
            .unwrap();
        let mut parent_polygon = Polygon::new_with_provider(
            Box::new(MockPolygonProvider {
                mock: Shared::clone(&mock),
            }),
            mesh.clone(),
        );

        let child_polygon = Shared::new(Polygon::new_with_provider(
            Box::new(MockPolygonProvider {
                mock: Shared::clone(&mock),
            }),
            mesh,
        ));
        parent_polygon.add_child(&child_polygon);

        // transform
        // assert_eq!()
    }
}
