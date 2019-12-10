#![feature(allocator_api, raw, asm)]

use linked_list_allocator::Heap;

use futures_task::noop_waker;
use futures_util::future::{pending, ready, FutureExt};

use pin_utils::pin_mut;

// use futures_core;

use core::alloc::{Alloc, Layout};
use core::future::Future;
use core::marker::PhantomData;
use core::mem;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::ptr;
use core::ptr::NonNull;
use core::raw::TraitObject;
use core::task::Context;

static mut FUT_HEAP_MEM: [u8; 1024] = [0; 1024];

static mut FUT_HEAP1: Heap = Heap::empty();

pub fn fut_box_oom(_layout: Layout) -> ! {
    // *TODO* find a way to say which pool this happened in
    panic!("ran out of memory");
}

pub struct FutBox<T: ?Sized> {
    ptr: NonNull<T>,
    // take over the ownership of T
    phantom: PhantomData<T>,
    raw_alloc_trait_object: TraitObject,
}

impl<T> FutBox<T> {
    pub fn new<A: Alloc>(x: T, a: &mut A) -> FutBox<T> {
        let layout = Layout::for_value(&x);
        let size = layout.size();
        let ptr = if size == 0 {
            NonNull::<T>::dangling()
        } else {
            unsafe {
                println!("alloc called on {:?}", layout);
                let ptr = a.alloc(layout).unwrap_or_else(|_| fut_box_oom(layout));
                ptr.cast()
            }
        };
        unsafe {
            ptr::write(ptr.as_ptr() as *mut T, x);
        }

        let alloc_trait_object: &dyn Alloc = a;
        let raw_alloc_trait_object: TraitObject = unsafe { mem::transmute(alloc_trait_object) };

        FutBox {
            ptr: ptr,
            phantom: PhantomData,
            raw_alloc_trait_object: raw_alloc_trait_object,
        }
    }
}

impl<T: ?Sized> Deref for FutBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for FutBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T: ?Sized> Drop for FutBox<T> {
    fn drop(&mut self) {
        unsafe {
            let layout = Layout::for_value(self.ptr.as_ref());
            ptr::drop_in_place(self.ptr.as_ptr());
            if layout.size() != 0 {
                let synthesized_alloc: &mut dyn Alloc = mem::transmute(TraitObject {
                    data: self.raw_alloc_trait_object.data,
                    vtable: self.raw_alloc_trait_object.vtable,
                });
                println!("dealloc called on {:?}", layout);
                synthesized_alloc.dealloc(self.ptr.cast(), layout);
            }
        }
    }
}

async fn async_ready() -> u8 {
    let a = ready(1);
    a.await
}

struct DropMe {
    msg: &'static str,
}

impl Drop for DropMe {
    fn drop(&mut self) {
        println!("{}", self.msg);
    }
}

// async fn async_pending() -> u8 {
//     let _drop_me = DropMe {
//         msg: "async_pending dropped!",
//     };

//     let a = pending::<u8>();
//     a.await;

//     0
// }

fn async_pending() -> impl Future<Output=u8> {
    let _drop_me = DropMe {
        msg: "async_pending dropped!",
    };

    async {
        let a = pending::<u8>();
        a.await;

        0
    }
}

async fn async_drop_test() -> u8 {
    let a1 = async_ready().fuse();
    let a2 = async_pending().fuse();

    pin_mut!(a1, a2);

    futures_util::select_biased! {
        x1 = a1 => {
            x1
        },
        x2 = a2 => {
            // This will return pending and a1 will resolve
            x2
        },
    }
}

fn main() {
    unsafe {
        FUT_HEAP1.init(&FUT_HEAP_MEM[0] as *const u8 as usize, 1024);
    }

    // T : !Unpin + Future
    let async_ready_fut = async_ready();

    // Move into FutBox
    let async_ready_fut_box = FutBox::new(async_ready_fut, unsafe { &mut FUT_HEAP1 });

    let mut pinned_async_ready_fut_box = unsafe { Pin::new_unchecked(async_ready_fut_box) };

    let w = noop_waker();

    println!(
        "future returned {:?}",
        pinned_async_ready_fut_box
            .as_mut()
            .poll(&mut Context::from_waker(&w))
    );
    drop(pinned_async_ready_fut_box);

    println!("---");

    // T: !Unpin + Future
    let async_drop_test_fut = async_drop_test();

    // Move into FutBox
    let async_drop_test_fut_box = FutBox::new(async_drop_test_fut, unsafe { &mut FUT_HEAP1 });

    let mut pinned_async_drop_test_fut_box = unsafe { Pin::new_unchecked(async_drop_test_fut_box) };

    println!(
        "future returned {:?}",
        pinned_async_drop_test_fut_box
            .as_mut()
            .poll(&mut Context::from_waker(&w))
    );

    drop(pinned_async_drop_test_fut_box);

    println!("---");

    // T: !Unpin + Future
    let async_pending_test_fut = async_pending();

    // Move into FutBox
    let async_pending_test_fut_box = FutBox::new(async_pending_test_fut, unsafe { &mut FUT_HEAP1 });

    let mut pinned_async_drop_test_fut_box =
        unsafe { Pin::new_unchecked(async_pending_test_fut_box) };

    println!(
        "future returned {:?}",
        pinned_async_drop_test_fut_box
            .as_mut()
            .poll(&mut Context::from_waker(&w))
    );

    drop(pinned_async_drop_test_fut_box);

    println!("---");
}
