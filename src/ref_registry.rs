// use std::marker::PhantomData;
//
// #[derive(Debug)]
// pub struct VarRegistry<T: Sized> {
//     vars: Vec<T>
// }
//
// #[derive(Debug)]
// pub struct VarRef {
//     i: usize
// }
//
// impl <T: Sized> VarRegistry<T> {
//     pub fn new() -> Self {
//         Self { vars: Vec::new() }
//     }
//
//     pub fn with_capacity(capacity: usize) -> Self {
//         Self { vars: Vec::with_capacity(capacity) }
//     }
//
//     pub fn len(&self) -> usize {
//         self.vars.len()
//     }
//
//     pub fn add(&mut self, item: T) -> VarRef {
//         let ref_ = VarRef { i: self.vars.len() };
//         self.vars.push(item);
//         ref_
//     }
//
//     pub fn get(&self, ref_: VarRef) -> &T {
//         &self.vars[ref_.i]
//     }
// }


// use std::marker::PhantomData;
//
// #[derive(Debug)]
// pub struct RefRegistry<T: Sized> {
//     items: Vec<T>
// }
//
// #[derive(Debug)]
// pub struct Ref<T: Sized> {
//     i: usize,
//     _tag: PhantomData<T>
// }
//
// impl <T: Sized> RefRegistry<T> {
//     pub fn new() -> Self {
//         Self { items: Vec::new() }
//     }
//
//     pub fn with_capacity(capacity: usize) -> Self {
//         Self { items: Vec::with_capacity(capacity) }
//     }
//
//     pub fn len(&self) -> usize {
//         self.items.len()
//     }
//
//     pub fn add(&mut self, item: T) -> Ref<T> {
//         let ref_ = Ref { i: self.items.len(), _tag: PhantomData };
//         self.items.push(item);
//         ref_
//     }
//
//     pub fn get(&self, ref_: Ref<T>) -> &T {
//         &self.items[ref_.i]
//     }
// }
