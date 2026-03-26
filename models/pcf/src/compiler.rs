use utils::{Compiler, Machine};

use crate::{
    cek_machine::CekMachine,
    secd_machine::{SecdCode, SecdMachine, compile_expr},
    syntax::{ExprCode, PrintEffect},
};

pub struct ExprToCekCompiler;

impl Compiler for ExprToCekCompiler {
    type Source = crate::expr_machine::ExprMachine;
    type Target = CekMachine;

    fn compile(
        source: <<Self as Compiler>::Source as Machine>::Code,
    ) -> Result<<<Self as Compiler>::Target as Machine>::Code, String> {
        Ok(source)
    }

    fn encode_ainput(
        ainput: <<Self as Compiler>::Source as Machine>::AInput,
    ) -> Result<<<Self as Compiler>::Target as Machine>::AInput, String> {
        Ok(ainput)
    }

    fn encode_rinput(
        rinput: <<Self as Compiler>::Source as Machine>::RInput,
    ) -> Result<<<Self as Compiler>::Target as Machine>::RInput, String> {
        Ok(rinput)
    }

    fn decode_routput(
        output: <<Self as Compiler>::Target as Machine>::ROutput,
    ) -> Result<<<Self as Compiler>::Source as Machine>::ROutput, String> {
        Ok(output)
    }

    fn decode_foutput(
        output: <<Self as Compiler>::Target as Machine>::FOutput,
    ) -> Result<<<Self as Compiler>::Source as Machine>::FOutput, String> {
        Ok(output)
    }
}

pub struct ExprToSecdCompiler;

impl Compiler for ExprToSecdCompiler {
    type Source = crate::expr_machine::ExprMachine;
    type Target = SecdMachine;

    fn compile(
        source: <<Self as Compiler>::Source as Machine>::Code,
    ) -> Result<<<Self as Compiler>::Target as Machine>::Code, String> {
        let ExprCode(expr) = source;
        Ok(SecdCode(compile_expr(&expr)))
    }

    fn encode_ainput(
        ainput: <<Self as Compiler>::Source as Machine>::AInput,
    ) -> Result<<<Self as Compiler>::Target as Machine>::AInput, String> {
        Ok(ainput)
    }

    fn encode_rinput(
        rinput: <<Self as Compiler>::Source as Machine>::RInput,
    ) -> Result<<<Self as Compiler>::Target as Machine>::RInput, String> {
        Ok(rinput)
    }

    fn decode_routput(
        output: <<Self as Compiler>::Target as Machine>::ROutput,
    ) -> Result<<<Self as Compiler>::Source as Machine>::ROutput, String> {
        let PrintEffect(effect) = output;
        Ok(PrintEffect(effect))
    }

    fn decode_foutput(
        output: <<Self as Compiler>::Target as Machine>::FOutput,
    ) -> Result<<<Self as Compiler>::Source as Machine>::FOutput, String> {
        Ok(output)
    }
}
