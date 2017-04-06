use super::iunknown::*;
use std::ops::Deref;
use std::ptr;

pub struct ComPtr<T: ComInterface> {
    instance: *const T
}

impl<T: ComInterface> ComPtr<T> {
    pub unsafe fn from_raw(instance: *const T) -> ComPtr<T> {
        println!("creating");
        // TODO: check if pointer is null
        ComPtr { instance: instance }
    }
}

impl<T: ComInterface> Drop for ComPtr<T> {
    fn drop(&mut self) {
        println!("dropping");
        let temp = self.instance;
        if !self.instance.is_null() {
            self.instance = ptr::null();
            unsafe {
                let unk = (&*temp).as_ref();
                println!("release");
                unk.release();
            }
        }
    }
}

impl<T: ComInterface> Deref for ComPtr<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.instance }
    }
}

impl<T: ComInterface> Clone for ComPtr<T> {
    fn clone(&self) -> Self {
        let unk = self.as_ref();
        unsafe {
            println!("add ref");
            unk.add_ref();
        }

        ComPtr { instance: self.instance }
    }
}

unsafe impl<T: ComInterface> Send for ComPtr<T> {
}
