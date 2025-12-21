use utils::Compiler;

pub struct Asm;

impl Compiler for Asm {
    type Source = crate::Environment;
    type Target = tiny_isa::Environment;

    fn compile(
        source: <<Self as Compiler>::Source as utils::Machine>::Code,
    ) -> Result<<<Self as Compiler>::Target as utils::Machine>::Code, String> {
        todo!()
    }

    fn encode_ainput(
        ainput: <<Self as Compiler>::Source as utils::Machine>::AInput,
    ) -> Result<<<Self as Compiler>::Target as utils::Machine>::AInput, String> {
        todo!()
    }

    fn encode_rinput(
        rinput: <<Self as Compiler>::Source as utils::Machine>::RInput,
    ) -> Result<<<Self as Compiler>::Target as utils::Machine>::RInput, String> {
        todo!()
    }

    fn decode_output(
        output: <<Self as Compiler>::Target as utils::Machine>::Output,
    ) -> Result<<<Self as Compiler>::Source as utils::Machine>::Output, String> {
        todo!()
    }
}
