#![doc = include_str!("../README.md")]

use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    marker::PhantomData,
};

/// Generational allocations span.
#[derive(Default)]
pub struct Span(Vec<Alloc>);

impl Span {
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocates `v` on the heap and stores the pointer in a generational allocation.
    ///
    /// Then the [`Span`] gets dropped it recycles it's generational allocations so that they
    /// can be reused by other spans.
    ///
    /// The returned [`Ptr<T>`] is [`Copy`] even if the underlying `T` is not [`Copy`].
    /// This pointer gets invalidated whenever it's [`Span`] is dropped.
    #[must_use]
    pub fn alloc<T: 'static>(&mut self, v: T) -> Ptr<T> {
        let alloc = RECYCLED_ALLOCS
            .with(|recycled| recycled.borrow_mut().pop())
            .unwrap_or_default();
        self.0.push(alloc);
        *alloc.ptr.borrow_mut() = Some(Box::new(v));
        Ptr {
            gen: alloc.gen,
            alloc,
            _marker: PhantomData,
        }
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        RECYCLED_ALLOCS.with(|recycled| {
            let mut recycled = recycled.borrow_mut();
            for alloc in &mut self.0 {
                let _ = alloc.ptr.take();
                alloc.gen += 1;
                recycled.push(*alloc)
            }
        });
    }
}

/// Generational pointer.
///
/// [`Ptr<T>`] is [`Copy`] even if the underlying `T` is not [`Copy`].
pub struct Ptr<T> {
    alloc: Alloc,
    gen: u32,
    _marker: PhantomData<T>,
}

impl<T> Clone for Ptr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Ptr<T> {}

impl<T> Ptr<T> {
    pub fn read(&self) -> Ref<'static, T> {
        assert_eq!(self.gen, self.alloc.gen);
        let borrow = self.alloc.ptr.borrow();
        Ref::filter_map(borrow, |any| any.as_ref()?.downcast_ref()).unwrap()
    }

    pub fn write(&self) -> RefMut<'static, T> {
        assert_eq!(self.gen, self.alloc.gen);
        let borrow = self.alloc.ptr.borrow_mut();
        RefMut::filter_map(borrow, |any| any.as_mut()?.downcast_mut()).unwrap()
    }
}

/// Generational allocation.
#[derive(Clone, Copy)]
struct Alloc {
    ptr: &'static RefCell<Option<Box<dyn Any>>>,
    gen: u32,
}

impl Default for Alloc {
    fn default() -> Self {
        Self {
            ptr: &*Box::leak(Default::default()),
            gen: 0,
        }
    }
}

thread_local! {
    static RECYCLED_ALLOCS: RefCell<Vec<Alloc>> = const {
        RefCell::new(Vec::new())
    };
}

#[test]
fn ptr_is_copy() {
    let mut span = Span::new();
    let ptr_1 = span.alloc("test".to_string());
    let ptr_2 = ptr_1;
    assert_eq!(*ptr_1.read(), *ptr_2.read());
}
