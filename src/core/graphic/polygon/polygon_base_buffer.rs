use crate::core::graphic::polygon::polygon_base::PolygonBase;
use crate::utility::buffer_interface::BufferInterface;
use nalgebra_glm::vec3_to_vec4;

pub trait PolygonBaseBuffer<TBuffer: BufferInterface<f32>>: PolygonBase {
    fn copy_positions_into(&self, buffer: &mut TBuffer, offset: u16) {
        let change_range = self.position_change_range();
        if let Some(range) = change_range.get_range() {
            let matrix = self.transform();
            let mesh_positions = &self.mesh().positions;
            let position = self.position();
            if self.computed_visible() {
                for i in range {
                    let p = position + &mesh_positions[i as usize];
                    let v = &matrix * vec3_to_vec4(&p);
                    buffer.copy(offset + i * 3 + 0, v.x);
                    buffer.copy(offset + i * 3 + 1, v.y);
                    buffer.copy(offset + i * 3 + 2, v.z);
                }
            } else {
                for i in range {
                    buffer.copy(offset + i * 3 + 0, 0.0f32);
                    buffer.copy(offset + i * 3 + 1, 0.0f32);
                    buffer.copy(offset + i * 3 + 2, 0.0f32);
                }
            }
        }
    }

    fn copy_colors_into(&self, buffer: &mut TBuffer, offset: u16) {
        let change_range = self.color_change_range();
        let parent = self.parent();
        if let Some(range) = change_range.get_range() {
            let color = if let Some(x) = &parent {
                x.color()
            } else {
                self.color()
            };
            let mesh_colors = &self.mesh().colors;
            for i in range {
                let base_color = &mesh_colors[i as usize];
                buffer.copy(offset + i * 4 + 0, color.x * base_color.x);
                buffer.copy(offset + i * 4 + 1, color.y * base_color.y);
                buffer.copy(offset + i * 4 + 2, color.z * base_color.z);
                buffer.copy(offset + i * 4 + 3, color.w * base_color.w);
            }
        }
    }

    fn copy_texcoords_into(&self, buffer: &mut TBuffer, offset: u16) {
        let change_range = self.texcoord_change_range();
        if let Some(range) = change_range.get_range() {
            let mesh_texcoords = &self.mesh().texcoords;
            for i in range {
                let uv = &mesh_texcoords[i as usize];
                buffer.copy(offset + i * 2 + 0, uv.x);
                buffer.copy(offset + i * 2 + 1, uv.y);
            }
        }
    }

    fn copy_normals_into(&self, buffer: &mut TBuffer, offset: u16) {
        let change_range = self.normal_change_range();
        if let Some(range) = change_range.get_range() {
            let matrix = self.transform();
            let mesh_normals = &self.mesh().normals;
            for i in range {
                let v = &matrix * vec3_to_vec4(&mesh_normals[i as usize]);
                buffer.copy(offset + i * 3 + 0, v.x);
                buffer.copy(offset + i * 3 + 1, v.y);
                buffer.copy(offset + i * 3 + 2, v.z);
            }
        }
    }
}
