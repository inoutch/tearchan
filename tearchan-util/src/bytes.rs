use nalgebra_glm::{Alloc, DefaultAllocator, Dimension, RealField, TVec, TVec1};
use std::mem::size_of;

pub fn vec_to_bytes<N: RealField, D: Dimension>(x: &[TVec<N, D>]) -> &[u8]
where
    DefaultAllocator: Alloc<N, D>,
{
    unsafe {
        std::slice::from_raw_parts(x.as_ptr() as *const u8, x.len() * size_of::<TVec<N, D>>())
    }
}

pub fn flatten<N: RealField, D: Dimension>(x: &[TVec<N, D>]) -> &[N]
where
    DefaultAllocator: Alloc<N, D>,
{
    unsafe {
        std::slice::from_raw_parts(
            x.as_ptr() as *const N,
            x.len() * size_of::<TVec<N, D>>() / size_of::<TVec1<N>>(),
        )
    }
}

#[cfg(test)]
mod test {
    use crate::bytes::{flatten, vec_to_bytes};
    use endianness::{read_f32, ByteOrder};
    use nalgebra_glm::vec2;

    #[test]
    fn test_vec_to_bytes() {
        let data = vec![
            vec2(27342f32, -1837842f32),
            vec2(9782323f32, 0.2847454f32),
            vec2(std::f32::MAX, std::f32::MIN),
        ];
        let slice = vec_to_bytes(&data);
        assert_eq!(slice.len(), data.len() * 4 * 2);
        assert_eq!(
            read_f32(&slice[0..4], ByteOrder::LittleEndian).unwrap(),
            27342f32
        );
        assert_eq!(
            read_f32(&slice[4..8], ByteOrder::LittleEndian).unwrap(),
            -1837842f32
        );
        assert_eq!(
            read_f32(&slice[8..12], ByteOrder::LittleEndian).unwrap(),
            9782323f32
        );
        assert_eq!(
            read_f32(&slice[12..16], ByteOrder::LittleEndian).unwrap(),
            0.2847454f32
        );
        assert_eq!(
            read_f32(&slice[16..20], ByteOrder::LittleEndian).unwrap(),
            std::f32::MAX
        );
        assert_eq!(
            read_f32(&slice[20..24], ByteOrder::LittleEndian).unwrap(),
            std::f32::MIN
        );
    }

    #[test]
    fn test_flatten() {
        let data = vec![
            vec2(27342f32, -1837842f32),
            vec2(9782323f32, 0.2847454f32),
            vec2(std::f32::MAX, std::f32::MIN),
        ];
        let flat: &[f32] = flatten(&data);
        assert_eq!(flat.len(), 6);

        assert_eq!(flat[0], 27342f32);
        assert_eq!(flat[1], -1837842f32);
        assert_eq!(flat[2], 9782323f32);
        assert_eq!(flat[3], 0.2847454f32);
        assert_eq!(flat[4], std::f32::MAX);
        assert_eq!(flat[5], std::f32::MIN);
    }
}
