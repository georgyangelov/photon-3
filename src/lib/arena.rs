use std::marker::PhantomData;

pub struct Arena<T: Sized> {
    values: Vec<T>
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct ArenaRef<T: Sized> {
    i: usize,
    refers_to: PhantomData<T>
}

impl <T: Sized> Arena<T> {
    // TODO: Optimize by giving it capacity from the start
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn allocate(&mut self, value: T) -> ArenaRef<T> {
        let i = self.values.len();

        self.values.push(value);

        ArenaRef { i, refers_to: PhantomData }
    }

    pub fn get(&self, arena_ref: ArenaRef<T>) -> &T {
        &self.values[arena_ref.i]
    }

    pub fn set(&mut self, arena_ref: ArenaRef<T>, value: T) {
        self.values[arena_ref.i] = value
    }

    pub fn map<R>(self, mapper: fn(T) -> R) -> Arena<R> {
        Arena { values: self.values.into_iter().map(mapper).collect() }
    }
}