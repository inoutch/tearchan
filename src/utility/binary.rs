use std::ptr;

pub unsafe fn get_value_from_ptr<T>(bytes_ptr: *const u8, offset: usize, default: T) -> T
where
    T: Copy,
{
    let binary_offset = offset * std::mem::size_of::<T>();
    let mut ret: T = default;
    ptr::copy_nonoverlapping(
        bytes_ptr.add(binary_offset),
        &mut ret as *mut T as *mut u8,
        std::mem::size_of::<T>(),
    );
    ret
}

pub unsafe fn set_value_to_ptr<T>(bytes_ptr: *mut u8, offset: usize, value: T)
where
    T: Copy,
{
    let binary_offset = offset * std::mem::size_of::<T>();
    ptr::copy_nonoverlapping(
        &value as *const T as *const u8,
        bytes_ptr.add(binary_offset),
        std::mem::size_of::<T>(),
    );
}

#[cfg(test)]
mod test {
    use crate::utility::binary::{set_value_to_ptr, get_value_from_ptr};

    #[test]
    fn test_get() {
        let mut bytes = vec![0u8; 128];
        unsafe { set_value_to_ptr(bytes.as_mut_ptr(), 0, 256u32) }

        assert_eq!(bytes[0], 0u8);
        assert_eq!(bytes[1], 1u8);
        assert_eq!(bytes[2], 0u8);
        assert_eq!(bytes[3], 0u8);

        assert_eq!(unsafe { get_value_from_ptr(bytes.as_ptr(), 0, 0) }, 256);

        unsafe { set_value_to_ptr(bytes.as_mut_ptr(), 1, 182746573u32) }

        assert_eq!(unsafe { get_value_from_ptr(bytes.as_ptr(), 0, 0) }, 256);
        assert_eq!(unsafe { get_value_from_ptr(bytes.as_ptr(), 1, 0) }, 182746573u32);
        assert_eq!(unsafe { get_value_from_ptr(bytes.as_ptr(), 2, 0) }, 0);
    }
}
