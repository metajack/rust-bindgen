/* automatically generated by rust-bindgen */


#![feature(const_fn)]
#![allow(non_snake_case)]


#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Enum_enum_and_vtable_mangling_hpp_unnamed_1 {
    match_ = 0,
    whatever_else = 1,
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct Struct_C {
    pub _vftable: *const _vftable_Struct_C,
    pub i: ::std::os::raw::c_int,
}
#[repr(C)]
pub struct _vftable_Struct_C {
    pub match_: unsafe extern "C" fn(this: *mut ::std::os::raw::c_void),
}
impl ::std::clone::Clone for Struct_C {
    fn clone(&self) -> Self { *self }
}
#[test]
fn bindgen_test_layout_Struct_C() {
    assert_eq!(::std::mem::size_of::<Struct_C>() , 16usize);
    assert_eq!(::std::mem::align_of::<Struct_C>() , 8usize);
}
