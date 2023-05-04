use crate::{implicit, melody};

pub trait Allocator<T> {
    type Holder;
    type Several;

    fn as_ref(holder: &Self::Holder) -> &T;
    fn as_slice(several: &Self::Several) -> &[T];

    fn pack(&mut self, value: T) -> Self::Holder;
    fn pack_many<I>(&mut self, values: I) -> Self::Several
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator;
}

#[derive(Clone, Copy, Debug)]
pub struct Heap;

impl<N, Id> Allocator<implicit::Melody<N, Id, Self>> for Heap {
    type Holder = Box<implicit::Melody<N, Id, Self>>;
    type Several = Vec<implicit::Melody<N, Id, Self>>;

    fn as_ref(holder: &Self::Holder) -> &implicit::Melody<N, Id, Self> {
        holder.as_ref()
    }

    fn as_slice(several: &Self::Several) -> &[implicit::Melody<N, Id, Self>] {
        several.as_slice()
    }

    fn pack(&mut self, value: implicit::Melody<N, Id, Self>) -> Self::Holder {
        Self::Holder::new(value)
    }

    fn pack_many<I>(&mut self, values: I) -> Vec<implicit::Melody<N, Id, Self>>
    where
        I: IntoIterator<Item = implicit::Melody<N, Id, Self>>,
        I::IntoIter: ExactSizeIterator,
    {
        values.into_iter().collect()
    }
}

impl<N, Id> Allocator<melody::Melody<N, Id, Self>> for Heap {
    type Holder = Box<melody::Melody<N, Id, Self>>;
    type Several = Vec<melody::Melody<N, Id, Self>>;

    fn as_ref(holder: &Self::Holder) -> &melody::Melody<N, Id, Self> {
        holder.as_ref()
    }

    fn as_slice(several: &Self::Several) -> &[melody::Melody<N, Id, Self>] {
        several.as_slice()
    }

    fn pack(&mut self, value: melody::Melody<N, Id, Self>) -> Self::Holder {
        Self::Holder::new(value)
    }

    fn pack_many<I>(&mut self, values: I) -> Self::Several
    where
        I: IntoIterator<Item = melody::Melody<N, Id, Self>>,
        I::IntoIter: ExactSizeIterator,
    {
        values.into_iter().collect()
    }
}

pub struct Arena<'a, N, Id> {
    implicits: typed_arena::Arena<implicit::Melody<N, Id, &'a Self>>,
    explicits: typed_arena::Arena<melody::Melody<N, Id, &'a Self>>,
}

impl<'a, N, Id> Arena<'a, N, Id> {
    pub fn new() -> Self {
        Self {
            implicits: typed_arena::Arena::new(),
            explicits: typed_arena::Arena::new(),
        }
    }
}

impl<'a, N, Id> Default for Arena<'a, N, Id> {
    fn default() -> Self {
        Self {
            implicits: typed_arena::Arena::default(),
            explicits: typed_arena::Arena::default(),
        }
    }
}

impl<'a, N, Id> Allocator<implicit::Melody<N, Id, Self>> for &'a Arena<'a, N, Id> {
    type Holder = &'a implicit::Melody<N, Id, Self>;
    type Several = &'a [implicit::Melody<N, Id, Self>];

    fn as_ref(holder: &Self::Holder) -> &implicit::Melody<N, Id, Self> {
        holder
    }

    fn as_slice(several: &Self::Several) -> &[implicit::Melody<N, Id, Self>] {
        several
    }

    fn pack(&mut self, value: implicit::Melody<N, Id, Self>) -> Self::Holder {
        self.implicits.alloc(value)
    }

    fn pack_many<I>(&mut self, values: I) -> Self::Several
    where
        I: IntoIterator<Item = implicit::Melody<N, Id, Self>>,
        I::IntoIter: ExactSizeIterator,
    {
        self.implicits.alloc_extend(values)
    }
}

impl<'a, N, Id> Allocator<melody::Melody<N, Id, Self>> for &'a Arena<'a, N, Id> {
    type Holder = &'a melody::Melody<N, Id, Self>;
    type Several = &'a [melody::Melody<N, Id, Self>];

    fn as_ref(holder: &Self::Holder) -> &melody::Melody<N, Id, Self> {
        holder
    }

    fn as_slice(several: &Self::Several) -> &[melody::Melody<N, Id, Self>] {
        several
    }

    fn pack(&mut self, value: melody::Melody<N, Id, Self>) -> Self::Holder {
        self.explicits.alloc(value)
    }

    fn pack_many<I>(&mut self, values: I) -> Self::Several
    where
        I: IntoIterator<Item = melody::Melody<N, Id, Self>>,
        I::IntoIter: ExactSizeIterator,
    {
        self.explicits.alloc_extend(values)
    }
}
