#![feature(allocator_api)]
#![feature(nonnull_slice_from_raw_parts)]

use core::{
    alloc::{AllocError, Allocator, Layout},
    ffi::{c_uint, c_void},
    marker::PhantomData,
    ptr::{self, NonNull},
};

use esp_idf_sys::{
    heap_caps_aligned_alloc, heap_caps_aligned_calloc, heap_caps_calloc, heap_caps_free,
    heap_caps_malloc, heap_caps_realloc, MALLOC_CAP_32BIT, MALLOC_CAP_DMA, MALLOC_CAP_INTERNAL,
    MALLOC_CAP_SPIRAM,
};

pub struct HeapCapsAlloc<T: Caps> {
    _t: PhantomData<T>,
}

pub unsafe trait Caps {
    const CAPS: c_uint;
}

pub struct Aligned32Bit;
unsafe impl Caps for Aligned32Bit {
    const CAPS: c_uint = MALLOC_CAP_32BIT;
}

pub struct Dma;
unsafe impl Caps for Dma {
    const CAPS: c_uint = MALLOC_CAP_DMA;
}

pub struct Internal;
unsafe impl Caps for Internal {
    const CAPS: c_uint = MALLOC_CAP_INTERNAL;
}

pub struct SpiRam;
unsafe impl Caps for SpiRam {
    const CAPS: c_uint = MALLOC_CAP_SPIRAM;
}

pub const ALLOC_ALIGNED_32BIT: HeapCapsAlloc<Aligned32Bit> = HeapCapsAlloc { _t: PhantomData };
pub const ALLOC_DMA: HeapCapsAlloc<Dma> = HeapCapsAlloc { _t: PhantomData };
pub const ALLOC_INTERNAL: HeapCapsAlloc<Internal> = HeapCapsAlloc { _t: PhantomData };
pub const ALLOC_SPIRAM: HeapCapsAlloc<SpiRam> = HeapCapsAlloc { _t: PhantomData };

unsafe impl<T: Caps> Allocator for HeapCapsAlloc<T> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = if layout.align() > 8 {
            unsafe { heap_caps_aligned_alloc(layout.align(), layout.size(), T::CAPS) }
        } else {
            unsafe { heap_caps_malloc(layout.size(), T::CAPS) }
        };
        let ptr = NonNull::new(ptr as *mut u8).ok_or(AllocError)?;
        Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
    }

    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = if layout.align() > 8 {
            unsafe { heap_caps_aligned_calloc(layout.align(), layout.size(), 1, T::CAPS) }
        } else {
            unsafe { heap_caps_calloc(layout.size(), 1, T::CAPS) }
        };
        let ptr = NonNull::new(ptr as *mut u8).ok_or(AllocError)?;
        Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, _layout: Layout) {
        heap_caps_free(ptr.as_ptr() as *mut c_void);
    }

    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        _old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = heap_caps_realloc(ptr.as_ptr() as *mut c_void, new_layout.size(), T::CAPS);
        let ptr = NonNull::new(ptr as *mut u8).ok_or(AllocError)?;
        Ok(NonNull::slice_from_raw_parts(ptr, new_layout.size()))
    }

    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = self.grow(ptr, old_layout, new_layout)?;
        ptr::write_bytes(
            (ptr.as_ptr() as *mut u8).add(old_layout.size()),
            0,
            new_layout.size() - old_layout.size(),
        );
        Ok(ptr)
    }

    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        _old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = heap_caps_realloc(ptr.as_ptr() as *mut c_void, new_layout.size(), T::CAPS);
        let ptr = NonNull::new(ptr as *mut u8).ok_or(AllocError)?;
        Ok(NonNull::slice_from_raw_parts(ptr, new_layout.size()))
    }
}
