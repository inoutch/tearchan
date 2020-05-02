use crate::core::graphic::polygon::polygon_base::PolygonBase;
use crate::utility::buffer_interface::BufferInterface;
use nalgebra_glm::vec3_to_vec4;

pub trait PolygonBaseBuffer<TBuffer: BufferInterface<f32>>: PolygonBase {
    fn copy_positions_into(&mut self, buffer: &mut TBuffer, offset: usize) {
        let change_range = self.position_change_range();
        if let Some(range) = change_range.get_range() {
            let matrix = self.transform();
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
            self.reset_all_position_change_range();
        }
    }

    fn copy_colors_into(&mut self, buffer: &mut TBuffer, offset: usize) {
        let change_range = self.color_change_range();
        let parent = self.parent();
        if let Some(range) = change_range.get_range() {
            let color = if let Some(x) = &parent {
                x.color()
            } else {
                self.color()
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
            self.reset_all_color_change_range();
        }
    }

    fn copy_texcoords_into(&mut self, buffer: &mut TBuffer, offset: usize) {
        let change_range = self.texcoord_change_range();
        if let Some(range) = change_range.get_range() {
            let mesh_texcoords = &self.mesh().texcoords;

            buffer.update_with_range(range.start * 2, range.end * 2);
            for i in range {
                let uv = &mesh_texcoords[i as usize];
                buffer.copy(offset + i * 2, uv.x);
                buffer.copy(offset + i * 2 + 1, uv.y);
            }
            self.reset_all_texcoord_change_range();
        }
    }

    fn copy_normals_into(&mut self, buffer: &mut TBuffer, offset: usize) {
        let change_range = self.normal_change_range();
        if let Some(range) = change_range.get_range() {
            let matrix = self.transform();
            let mesh_normals = &self.mesh().normals;

            buffer.update_with_range(range.start * 3, range.end * 3);
            for i in range {
                let m = &matrix;
                let v = m * vec3_to_vec4(&mesh_normals[i as usize]);
                buffer.copy(offset + i * 3, v.x);
                buffer.copy(offset + i * 3 + 1, v.y);
                buffer.copy(offset + i * 3 + 2, v.z);
            }
            self.reset_all_normal_change_range();
        }
    }
}
