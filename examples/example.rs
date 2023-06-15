#[repr(C)]
#[derive(Clone, Copy)]
struct TwoNumbs {
    pub a: i32,
    pub b: i32,
}

#[no_mangle]
extern "C" fn canonical_abi_realloc(ptr: usize, old_size: usize, align: usize, new_size: usize) -> *mut u8 {
    if ptr == 0 {
        let new_layout = std::alloc::Layout::from_size_align(new_size, align).unwrap();
        return unsafe { std::alloc::alloc(new_layout) };
    }
    let new_layout = std::alloc::Layout::from_size_align(new_size, align).unwrap();
    unsafe { std::alloc::realloc(ptr as *mut u8, new_layout, new_size) }
}

#[no_mangle]
extern "C" fn canonical_abi_free(ptr: usize, old_size: usize, align: usize) {
    let old_layout = std::alloc::Layout::from_size_align(old_size, align).unwrap();
    unsafe { std::alloc::dealloc(ptr as *mut u8, old_layout) }
}

#[no_mangle]
extern "C" fn sum_vec(vec: *mut TwoNumbs, len: usize) -> i32 {
    let mut sum = 0;
    for i in 0..len {
        let item = unsafe { *vec.add(i) };
        sum += item.a + item.b;
    }
    sum
}

fn main() {}
