use crate::core::graphic::batch::batch_change_manager::BatchChangeNotifier;
use crate::core::graphic::hal::buffer_interface::BufferMappedMemoryInterface;
use crate::extension::shared::{Shared, WeakShared};
use crate::math::change_range::ChangeRange;
use crate::math::mesh::{IndexType, Mesh};
use crate::math::vec::{make_vec3_fill, make_vec3_zero, make_vec4_white};
use crate::utility::change_notifier::{ChangeNotifier, ChangeNotifierObject};
use nalgebra_glm::{rotate, scale, translate, vec3, vec3_to_vec4, vec4, Mat4, Vec2, Vec3, Vec4};
use std::any::Any;
use std::cell::RefCell;
use std::option::Option::Some;
use std::rc::Rc;

pub mod billboard;
pub mod polygon_2d;
pub mod sprite_atlas;

pub trait PolygonProvider {
    fn position<'a>(&self, core: &'a PolygonCore) -> &'a Vec3 {
        core.position()
    }

    fn set_position(&mut self, core: &mut PolygonCore, position: Vec3) -> bool {
        if core.set_position(position) {
            self.request_change(core);
            true
        } else {
            false
        }
    }

    fn color<'a>(&self, core: &'a PolygonCore) -> &'a Vec4 {
        core.color()
    }

    fn set_color(&mut self, core: &mut PolygonCore, color: Vec4) -> bool {
        if core.set_color(color) {
            self.request_change(core);
            true
        } else {
            false
        }
    }

    fn computed_color(&self, core: &PolygonCore) -> Vec4 {
        core.computed_color()
    }

    fn scale<'a>(&self, core: &'a PolygonCore) -> &'a Vec3 {
        core.scale()
    }

    fn set_scale(&mut self, core: &mut PolygonCore, scale: Vec3) -> bool {
        if core.set_scale(scale) {
            self.request_change(core);
            true
        } else {
            false
        }
    }

    fn rotation_axis<'a>(&self, core: &'a PolygonCore) -> &'a Vec3 {
        core.rotation_axis()
    }

    fn set_rotation_axis(&mut self, core: &mut PolygonCore, rotation_axis: Vec3) -> bool {
        if core.set_rotation_axis(rotation_axis) {
            self.request_change(core);
            true
        } else {
            false
        }
    }

    fn rotation_radian(&self, core: &PolygonCore) -> f32 {
        core.rotation_radian()
    }

    fn set_rotation_radian(&mut self, core: &mut PolygonCore, rotation_radian: f32) -> bool {
        if core.set_rotation_radian(rotation_radian) {
            self.request_change(core);
            true
        } else {
            false
        }
    }

    fn visible(&self, core: &PolygonCore) -> bool {
        core.visible()
    }

    fn set_visible(&mut self, core: &mut PolygonCore, visible: bool) -> bool {
        if core.set_visible(visible) {
            self.request_change(core);
            true
        } else {
            false
        }
    }

    fn computed_visible(&self, core: &PolygonCore) -> bool {
        core.computed_visible()
    }

    fn add_child(&mut self, core: &mut PolygonCore, polygon: &Shared<Polygon>) {
        core.add_child(polygon);
    }

    fn transform(&self, core: &PolygonCore) -> Mat4;

    fn transform_for_child(&self, core: &PolygonCore) -> Mat4;

    fn as_any_provider_mut(&mut self) -> &mut dyn Any;

    fn request_change(&mut self, core: &mut PolygonCore) {
        core.request_change();
    }
}

pub trait PolygonCommon {
    fn mesh(&self) -> &Mesh;
    fn position(&self) -> &Vec3;
    fn set_position(&mut self, position: Vec3) -> bool;
    fn color(&self) -> &Vec4;
    fn set_color(&mut self, color: Vec4) -> bool;
    fn computed_color(&self) -> Vec4;
    fn scale(&self) -> &Vec3;
    fn set_scale(&mut self, scale: Vec3) -> bool;
    fn rotation_axis(&self) -> &Vec3;
    fn set_rotation_axis(&mut self, rotation_axis: Vec3) -> bool;
    fn rotation_radian(&self) -> f32;
    fn set_rotation_radian(&mut self, rotation_radian: f32) -> bool;
    fn visible(&self) -> bool;
    fn set_visible(&mut self, visible: bool) -> bool;
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
        let index_change_range = ChangeRange::new(mesh.indices.len());
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
                index_change_range,
                position_change_range,
                color_change_range,
                texcoord_change_range,
                normal_change_range,
                notifier: None,
            },
            provider,
        }
    }

    #[inline]
    pub fn transform_for_child(&self) -> Mat4 {
        self.provider.transform_for_child(&self.core)
    }

    #[inline]
    pub fn transform(&self) -> Mat4 {
        self.provider.transform(&self.core)
    }

    pub fn copy_indices_into<TBuffer: BufferMappedMemoryInterface<IndexType>>(
        &mut self,
        buffer: &mut TBuffer,
        offset: usize,
        vertex_offset: IndexType,
        force: bool,
    ) {
        let range = match force {
            true => self.core.index_change_range.get_range_or_full(),
            false => match self.core.index_change_range.get_range() {
                Some(range) => range,
                None => return,
            },
        };
        for i in range {
            let index = self.mesh().indices[i];
            buffer.set(index + vertex_offset, i + offset);
        }
        self.core.reset_all_index_change_range();
    }

    pub fn copy_positions_into<TBuffer: BufferMappedMemoryInterface<f32>>(
        &mut self,
        buffer: &mut TBuffer,
        offset: usize,
        force: bool,
    ) {
        let change_range = &self.core.position_change_range;
        let range = match force {
            true => change_range.get_range_or_full(),
            false => match change_range.get_range() {
                Some(range) => range,
                None => return,
            },
        };
        let matrix = self.provider.transform(&self.core);
        let mesh_positions = &self.mesh().positions;

        if self.computed_visible() {
            for i in range {
                let mesh_position = &mesh_positions[i as usize];
                let m = &matrix;
                let v = m * vec4(mesh_position.x, mesh_position.y, mesh_position.z, 1.0f32);
                buffer.set(v.x, offset + i * 3);
                buffer.set(v.y, offset + i * 3 + 1);
                buffer.set(v.z, offset + i * 3 + 2);
            }
        } else {
            for i in range {
                buffer.set(0.0f32, offset + i * 3);
                buffer.set(0.0f32, offset + i * 3 + 1);
                buffer.set(0.0f32, offset + i * 3 + 2);
            }
        }
        self.core.reset_all_position_change_range();
    }

    pub fn copy_colors_into<TBuffer: BufferMappedMemoryInterface<f32>>(
        &mut self,
        buffer: &mut TBuffer,
        offset: usize,
        force: bool,
    ) {
        let change_range = &self.core.color_change_range;
        let range = match force {
            true => change_range.get_range_or_full(),
            false => match change_range.get_range() {
                Some(range) => range,
                None => return,
            },
        };
        let color = if let Some(x) = self.core.parent() {
            x.borrow().computed_color()
        } else {
            self.computed_color()
        };
        let mesh_colors = &self.mesh().colors;

        for i in range {
            let base_color = &mesh_colors[i as usize];
            buffer.set(color.x * base_color.x, offset + i * 4);
            buffer.set(color.y * base_color.y, offset + i * 4 + 1);
            buffer.set(color.z * base_color.z, offset + i * 4 + 2);
            buffer.set(color.w * base_color.w, offset + i * 4 + 3);
        }
        self.core.reset_all_color_change_range();
    }

    pub fn copy_texcoords_into<TBuffer: BufferMappedMemoryInterface<f32>>(
        &mut self,
        buffer: &mut TBuffer,
        offset: usize,
        force: bool,
    ) {
        let change_range = &self.core.texcoord_change_range;
        let range = match force {
            true => change_range.get_range_or_full(),
            false => match change_range.get_range() {
                Some(range) => range,
                None => return,
            },
        };
        let mesh_texcoords = &self.mesh().texcoords;

        for i in range {
            let uv = &mesh_texcoords[i as usize];
            buffer.set(uv.x, offset + i * 2);
            buffer.set(uv.y, offset + i * 2 + 1);
        }
        self.core.reset_all_texcoord_change_range();
    }

    pub fn copy_normals_into<TBuffer: BufferMappedMemoryInterface<f32>>(
        &mut self,
        buffer: &mut TBuffer,
        offset: usize,
        force: bool,
    ) {
        let change_range = &self.core.normal_change_range;
        let range = match force {
            true => change_range.get_range_or_full(),
            false => match change_range.get_range() {
                Some(range) => range,
                None => return,
            },
        };

        let matrix = self.provider.transform(&self.core);
        let mesh_normals = &self.mesh().normals;

        for i in range {
            let m = &matrix;
            let v = m * vec3_to_vec4(&mesh_normals[i as usize]);
            buffer.set(v.x, offset + i * 3);
            buffer.set(v.y, offset + i * 3 + 1);
            buffer.set(v.z, offset + i * 3 + 2);
        }
        self.core.reset_all_normal_change_range();
    }

    pub fn vertex_size(&self) -> usize {
        self.mesh().size()
    }

    pub fn index_size(&self) -> usize {
        self.mesh().indices.len()
    }

    pub fn provider_as_any(&self) -> &dyn Any {
        &self.provider
    }

    pub fn provider_as_any_mut(&mut self) -> &mut dyn Any {
        self.provider.as_any_provider_mut()
    }
}

impl ChangeNotifierObject<BatchChangeNotifier<Polygon>> for Polygon {
    fn set_change_notifier(&mut self, notifier: BatchChangeNotifier<Polygon>) {
        let mut n = notifier;
        n.request_change();
        self.core.notifier = Some(n);
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
    fn set_position(&mut self, position: Vec3) -> bool {
        self.provider.set_position(&mut self.core, position)
    }

    #[inline]
    fn color(&self) -> &Vec4 {
        self.provider.color(&self.core)
    }

    #[inline]
    fn set_color(&mut self, color: Vec4) -> bool {
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
    fn set_scale(&mut self, scale: Vec3) -> bool {
        self.provider.set_scale(&mut self.core, scale)
    }

    #[inline]
    fn rotation_axis(&self) -> &Vec3 {
        self.provider.rotation_axis(&self.core)
    }

    #[inline]
    fn set_rotation_axis(&mut self, rotation_axis: Vec3) -> bool {
        self.provider
            .set_rotation_axis(&mut self.core, rotation_axis)
    }

    #[inline]
    fn rotation_radian(&self) -> f32 {
        self.provider.rotation_radian(&self.core)
    }

    #[inline]
    fn set_rotation_radian(&mut self, rotation_radian: f32) -> bool {
        self.provider
            .set_rotation_radian(&mut self.core, rotation_radian)
    }

    #[inline]
    fn visible(&self) -> bool {
        self.provider.visible(&self.core)
    }

    #[inline]
    fn set_visible(&mut self, visible: bool) -> bool {
        self.provider.set_visible(&mut self.core, visible)
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

pub struct PolygonCore<TNotifier = Polygon> {
    mesh: Mesh,
    position: Vec3,
    color: Vec4,
    scale: Vec3,
    rotation_axis: Vec3,
    rotation_radian: f32,
    visible: bool,
    parent: Option<WeakShared<Polygon>>,
    children: Vec<Shared<Polygon>>,
    index_change_range: ChangeRange,
    position_change_range: ChangeRange,
    color_change_range: ChangeRange,
    texcoord_change_range: ChangeRange,
    normal_change_range: ChangeRange,
    notifier: Option<BatchChangeNotifier<TNotifier>>,
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

    pub fn reset_all_index_change_range(&mut self) {
        self.index_change_range.reset();
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

    pub fn update_positions_of_mesh(&mut self, positions: Vec<Vec3>) {
        self.mesh.positions = positions;
        self.update_all_positions();
    }

    pub fn update_colors_of_mesh(&mut self, colors: Vec<Vec4>) {
        self.mesh.colors = colors;
        self.update_all_colors();
    }

    pub fn update_texcoords_of_mesh(&mut self, texcoords: Vec<Vec2>) {
        self.mesh.texcoords = texcoords;
        self.update_all_texcoords();
    }

    pub fn update_normals_of_mesh(&mut self, normals: Vec<Vec3>) {
        self.mesh.normals = normals;
        self.update_all_normals();
    }

    pub fn request_change(&mut self) {
        if let Some(notifier) = &mut self.notifier {
            notifier.request_change();
        }
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
    fn set_position(&mut self, position: Vec3) -> bool {
        if position != self.position {
            self.position = position;
            self.update_all_positions();
            true
        } else {
            false
        }
    }

    #[inline]
    fn color(&self) -> &Vec4 {
        &self.color
    }

    #[inline]
    fn set_color(&mut self, color: Vec4) -> bool {
        if color != self.color {
            self.color = color;
            self.color_change_range.update_all();
            true
        } else {
            false
        }
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
    fn set_scale(&mut self, scale: Vec3) -> bool {
        if scale != self.scale {
            self.scale = scale;
            self.update_all_positions();
            true
        } else {
            false
        }
    }

    #[inline]
    fn rotation_axis(&self) -> &Vec3 {
        &self.rotation_axis
    }

    #[inline]
    fn set_rotation_axis(&mut self, rotation_axis: Vec3) -> bool {
        if rotation_axis != self.rotation_axis {
            self.rotation_axis = rotation_axis;
            self.update_all_positions();
            self.update_all_normals();
            true
        } else {
            false
        }
    }

    #[inline]
    fn rotation_radian(&self) -> f32 {
        self.rotation_radian
    }

    #[inline]
    #[allow(clippy::float_cmp)]
    fn set_rotation_radian(&mut self, rotation_radian: f32) -> bool {
        if self.rotation_radian != rotation_radian {
            self.rotation_radian = rotation_radian;
            self.update_all_positions();
            self.update_all_normals();
            true
        } else {
            false
        }
    }

    #[inline]
    fn visible(&self) -> bool {
        self.visible
    }

    #[inline]
    fn set_visible(&mut self, visible: bool) -> bool {
        if self.visible != visible {
            self.visible = visible;
            self.update_all_positions();
            true
        } else {
            false
        }
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
    fn transform(&self, core: &PolygonCore) -> Mat4 {
        core.transform(self)
    }

    fn transform_for_child(&self, core: &PolygonCore) -> Mat4 {
        core.transform_for_children(self)
    }

    fn as_any_provider_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod test {
    use crate::core::graphic::polygon::{Polygon, PolygonCommon, PolygonCore, PolygonProvider};
    use crate::extension::shared::{make_shared, Shared};
    use crate::math::mesh::MeshBuilder;
    use crate::utility::change_notifier::ChangeNotifier;
    use crate::utility::test::func::MockFunc;
    use nalgebra_glm::{vec2, vec3, vec4, Mat4, Vec3, Vec4};
    use std::any::Any;

    struct MockPolygonProvider {
        mock: Shared<MockFunc>,
    }

    impl PolygonProvider for MockPolygonProvider {
        fn position<'a>(&self, core: &'a PolygonCore) -> &'a Vec3 {
            self.mock.borrow_mut().call(vec!["position".to_string()]);
            core.position()
        }

        fn set_position(&mut self, core: &mut PolygonCore, position: Vec3) -> bool {
            self.mock
                .borrow_mut()
                .call(vec!["set_position".to_string()]);
            core.set_position(position)
        }

        fn color<'a>(&self, core: &'a PolygonCore) -> &'a Vec4 {
            self.mock.borrow_mut().call(vec!["color".to_string()]);
            core.color()
        }

        fn set_color(&mut self, core: &mut PolygonCore, color: Vec4) -> bool {
            self.mock.borrow_mut().call(vec!["set_color".to_string()]);
            core.set_color(color)
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

        fn set_scale(&mut self, core: &mut PolygonCore, scale: Vec3) -> bool {
            self.mock.borrow_mut().call(vec!["set_scale".to_string()]);
            core.set_scale(scale)
        }

        fn rotation_axis<'a>(&self, core: &'a PolygonCore) -> &'a Vec3 {
            self.mock.borrow_mut().call(vec!["color".to_string()]);
            core.rotation_axis()
        }

        fn set_rotation_axis(&mut self, core: &mut PolygonCore, rotation_axis: Vec3) -> bool {
            self.mock
                .borrow_mut()
                .call(vec!["set_rotation_axis".to_string()]);
            core.set_rotation_axis(rotation_axis)
        }

        fn rotation_radian<'a>(&self, core: &PolygonCore) -> f32 {
            self.mock
                .borrow_mut()
                .call(vec!["rotation_radian".to_string()]);
            core.rotation_radian()
        }

        fn set_rotation_radian(&mut self, core: &mut PolygonCore, rotation_radian: f32) -> bool {
            self.mock
                .borrow_mut()
                .call(vec!["set_rotation_radian".to_string()]);
            core.set_rotation_radian(rotation_radian)
        }

        fn visible(&self, core: &PolygonCore) -> bool {
            self.mock.borrow_mut().call(vec!["visible".to_string()]);
            core.visible()
        }

        fn set_visible(&mut self, core: &mut PolygonCore, visible: bool) -> bool {
            self.mock.borrow_mut().call(vec!["set_visible".to_string()]);
            core.set_visible(visible)
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

        fn as_any_provider_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn request_change(&mut self, core: &mut PolygonCore<Polygon>) {
            self.mock
                .borrow_mut()
                .call(vec!["request_change".to_string()]);

            if let Some(notifier) = &mut core.notifier {
                notifier.request_change();
            }
        }
    }

    #[test]
    fn test_init() {
        let mock = make_shared(MockFunc::new());
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
        let mock = make_shared(MockFunc::new());
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
    /*fn test_position() {
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
        let vertices = create_square_positions(&Rect2 {
            origin: vec2(0.0f32, 0.0f32),
            size: vec2(32.0f32, 64.0f32),
        })
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
    }*/

    /*#[test]
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
    }*/

    /*#[test]
    fn test_texcoord() {
        let mesh = MeshBuilder::new()
            .with_square(vec2(32.0f32, 64.0f32))
            .build()
            .unwrap();

        let mut polygon = Polygon::new(mesh);
        let mut buffer = MockBuffer::new(256);
        let expected_texcoords = create_square_texcoords(&rect2(0.0f32, 0.0f32, 1.0f32, 1.0f32))
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

        let mut polygon = Polygon::new(mesh);
        let mut buffer = MockBuffer::new(256);
        polygon.copy_normals_into(&mut buffer, 0);

        // Allow empty for 2d
        assert_eq!(buffer.get_changes().len(), 0);
    }*/
    #[test]
    fn test_tree() {
        let mock = make_shared(MockFunc::new());
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

        let child_polygon = make_shared(Polygon::new_with_provider(
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
