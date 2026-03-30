pub mod data_lang;
pub mod expr_lang;
pub mod fn_ptr_machine;
pub mod internal_ctrl;
pub mod mini_prog_machine;
pub mod proc_lang;

mod fn_ptr_parser;
mod fn_ptr_render;
mod mini_prog_parser;
mod mini_prog_render;

#[cfg(test)]
mod mini_prog_tests;
