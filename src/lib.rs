use std::ops::{Deref, DerefMut};

use wasmtime::{AsContextMut, AsContext};

#[repr(C)]
struct RawVec<T> {
    pub(crate) ptr: i32,
    pub(crate) len: u32,
    pub(crate) cap: u32,
    pub(crate) _marker: std::marker::PhantomData<T>,
}

pub struct WasmVec<'a, T, S> {
    buf: RawVec<T>,
    store: &'a mut wasmtime::Store<S>,
    realloc: wasmtime::TypedFunc<(i32, i32, i32, i32), i32>,
    memory: wasmtime::Memory,
}

unsafe impl<T: Send> Send for RawVec<T> {}
unsafe impl<T: Sync> Sync for RawVec<T> {}

impl<'a, T, S> WasmVec<'a, T, S> {
    pub fn new(instance: &'a wasmtime::Instance, store: &'a mut wasmtime::Store<S>) -> Self {
        let realloc = instance
            .get_typed_func::<(i32, i32, i32, i32), i32>(store.as_context_mut(), "canonical_abi_realloc")
            .unwrap();
        let memory = instance.get_memory(store.as_context_mut(), "memory").unwrap();
        Self {
            buf: RawVec {
                ptr: 0,
                len: 0,
                cap: 0,
                _marker: std::marker::PhantomData,
            },
            realloc,
            store,
            memory,
        }
    }

    pub fn grow(&mut self) {
        let new_cap = if self.buf.cap == 0 { 1 } else { self.buf.cap * 2 };
        let new_layout = std::alloc::Layout::array::<T>(new_cap.try_into().unwrap()).unwrap();
        let new_ptr = {
            self.realloc.call(
                self.store.as_context_mut(),
                (self.buf.ptr,
                self.buf.cap as i32,
                new_layout.align().try_into().unwrap(),
                new_layout.size().try_into().unwrap())
            ).unwrap()
        };
        self.buf.ptr = new_ptr;
        self.buf.cap = new_cap;
    }

    pub fn push(&mut self, value: T) {
        if self.buf.len == self.buf.cap {
            self.grow();
        }
        unsafe {
            let ptr = self.memory.data_ptr(&self.store.as_context()).add(self.buf.ptr as usize) as *mut T;
            ptr.add(self.buf.len as usize).write(value);
        }
        self.buf.len += 1;
    }

    pub fn len(&self) -> usize {
        self.buf.len as usize
    }

    pub fn into_raw_parts(self) -> (i32, u32, u32) {
        let ptr = self.buf.ptr;
        let len = self.buf.len;
        let cap = self.buf.cap;
        std::mem::forget(self);
        (ptr, len, cap)
    }
}

impl<'a, T, S> Deref for WasmVec<'a, T, S> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(
                self.memory.data_ptr(&self.store.as_context()).add(self.buf.ptr as usize) as *const T,
                self.buf.len as usize,
            )
        }
    }
}

impl<'a, T, S> DerefMut for WasmVec<'a, T, S> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.memory.data_ptr(&self.store.as_context()).add(self.buf.ptr as usize) as *mut T,
                self.buf.len as usize,
            )
        }
    }
}
