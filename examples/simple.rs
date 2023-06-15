use wasmtime::{AsContextMut};
use wasmtime_memory_ext::WasmVec;

#[repr(C)]
struct TwoNumbs {
    pub a: i32,
    pub b: i32,
}

fn main() {
    let engine = wasmtime::Engine::default();
    let mut store = wasmtime::Store::new(&engine, ());
    let module = wasmtime::Module::from_file(&engine, "examples/example.wasm").unwrap();
    let instance = wasmtime::Instance::new(store.as_context_mut(), &module, &[]).unwrap();
    let mut vec = WasmVec::<TwoNumbs, ()>::new(&instance, &mut store);

    vec.push(TwoNumbs { a: 1, b: 2 });
    vec.push(TwoNumbs { a: 3, b: 4 });
    assert_eq!(vec.len(), 2);
    assert_eq!(vec[0].a, 1);
    assert_eq!(vec[0].b, 2);
    assert_eq!(vec[1].a, 3);
    assert_eq!(vec[1].b, 4);

    let (ptr, len, _) = vec.into_raw_parts();
    let sum_fn = instance.get_typed_func::<(i32, i32), i32>(&mut store, "sum_vec").unwrap();
    let sum = sum_fn.call(&mut store, (ptr, len as i32)).unwrap();
    assert_eq!(sum, 10);
}
