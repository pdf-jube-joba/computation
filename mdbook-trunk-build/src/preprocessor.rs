use super::*;

pub struct Nop;

impl Nop {
    pub fn new() -> Nop {
        Nop
    }
}

impl Preprocessor for Nop {
    fn name(&self) -> &str {
        "nop-preprocessor"
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }

    fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book, Error> {
        unimplemented!()
    }
}