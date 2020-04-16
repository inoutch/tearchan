use crate::core::graphic::polygon::polygon::Polygon;
use crate::math::change_range::ChangeRange;
use crate::math::mesh::Mesh;
use nalgebra_glm::{scale, translate, Mat4, Vec3, Vec4};
use std::cell::RefCell;
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

    fn children(&self) -> &Vec<Rc<RefCell<dyn PolygonBase>>> {
        &self.get_polygon().children
    }

    fn add_child(&mut self, child: Rc<RefCell<dyn PolygonBase>>) {
        self.get_mut_polygon().children.push(child);
    }

    fn set_visible(&mut self, visible: bool) {
        self.get_mut_polygon().visible = visible;
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

        polygon.children().iter().for_each(|x| {
            x.borrow_mut().update_all_positions();
        });
    }

    fn update_all_colors(&mut self) {
        let polygon = self.get_mut_polygon();
        polygon.color_change_range.update_all();

        polygon.children().iter().for_each(|x| {
            x.borrow_mut().update_all_colors();
        });
    }

    fn update_all_texcoords(&mut self) {
        let polygon = self.get_mut_polygon();
        polygon.texcoord_change_range.update_all();

        polygon.children().iter().for_each(|x| {
            x.borrow_mut().update_all_texcoords();
        });
    }

    fn update_all_normals(&mut self) {
        let polygon = self.get_mut_polygon();
        polygon.normal_change_range.update_all();

        polygon.children().iter().for_each(|x| {
            x.borrow_mut().update_all_normals();
        });
    }
}
