use crate::core::graphic::polygon::default::Polygon;
use crate::math::change_range::ChangeRange;
use crate::math::mesh::Mesh;
use nalgebra_glm::{scale, translate, Mat4, Vec3, Vec4};
use std::rc::Rc;

pub trait PolygonBase {
    fn get_mut_polygon(&mut self) -> &mut Polygon;

    fn get_polygon(&self) -> &Polygon;

    fn parent(&self) -> Option<Rc<dyn PolygonBase>> {
        if let Some(x) = &self.get_polygon().parent {
            x.upgrade()
        } else {
            None
        }
    }

    fn children(&mut self) -> &mut Vec<Box<dyn PolygonBase>> {
        &mut self.get_mut_polygon().children
    }

    fn add_child(&mut self, child: Box<dyn PolygonBase>) {
        self.get_mut_polygon().children.push(child);
    }

    fn set_visible(&mut self, visible: bool) {
        self.get_mut_polygon().visible = visible;
        self.update_all_positions();
    }

    fn visible(&self) -> bool {
        self.get_polygon().visible
    }

    fn computed_visible(&self) -> bool {
        if let Some(parent) = self.parent() {
            parent.visible()
        } else {
            self.visible()
        }
    }

    fn position(&self) -> &Vec3 {
        &self.get_polygon().position
    }

    fn set_position(&mut self, position: Vec3) {
        let polygon = self.get_mut_polygon();
        polygon.position = position;
        polygon.position_change_range.update_all();
    }

    fn color(&self) -> &Vec4 {
        &self.get_polygon().color
    }

    fn set_color(&mut self, color: Vec4) {
        self.get_mut_polygon().color = color;
    }

    fn scale(&self) -> &Vec3 {
        &self.get_polygon().scale
    }

    fn set_scale(&mut self, scale: Vec3) {
        self.get_mut_polygon().scale = scale;
    }

    fn transform(&self) -> Mat4 {
        let current =
            translate(&Mat4::identity(), self.position()) * scale(&Mat4::identity(), self.scale());
        if let Some(x) = self.parent() {
            x.transform_for_children() * current
        } else {
            current
        }
    }

    fn transform_for_children(&self) -> Mat4 {
        self.transform()
    }

    fn mesh(&self) -> &Mesh {
        &self.get_polygon().mesh
    }

    fn position_change_range(&self) -> &ChangeRange {
        &self.get_polygon().position_change_range
    }

    fn color_change_range(&self) -> &ChangeRange {
        &self.get_polygon().color_change_range
    }

    fn texcoord_change_range(&self) -> &ChangeRange {
        &self.get_polygon().texcoord_change_range
    }

    fn normal_change_range(&self) -> &ChangeRange {
        &self.get_polygon().normal_change_range
    }

    fn update_all_positions(&mut self) {
        let polygon = self.get_mut_polygon();
        polygon.position_change_range.update_all();

        polygon.children().iter_mut().for_each(|x| {
            x.update_all_positions();
        });
    }

    fn update_all_colors(&mut self) {
        let polygon = self.get_mut_polygon();
        polygon.color_change_range.update_all();

        polygon.children().iter_mut().for_each(|x| {
            x.update_all_colors();
        });
    }

    fn update_all_texcoords(&mut self) {
        let polygon = self.get_mut_polygon();
        polygon.texcoord_change_range.update_all();

        polygon.children().iter_mut().for_each(|x| {
            x.update_all_texcoords();
        });
    }

    fn update_all_normals(&mut self) {
        let polygon = self.get_mut_polygon();
        polygon.normal_change_range.update_all();

        polygon.children().iter_mut().for_each(|x| {
            x.update_all_normals();
        });
    }

    fn reset_all_position_change_range(&mut self) {
        self.get_mut_polygon().position_change_range.reset();
    }

    fn reset_all_color_change_range(&mut self) {
        self.get_mut_polygon().color_change_range.reset();
    }

    fn reset_all_texcoord_change_range(&mut self) {
        self.get_mut_polygon().texcoord_change_range.reset();
    }

    fn reset_all_normal_change_range(&mut self) {
        self.get_mut_polygon().normal_change_range.reset();
    }
}
