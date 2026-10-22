use super::output_parsers;
use brasa_completion_metadata::{CommandBuilder, CommandSignatureGenerators, Generator};

pub fn generator() -> CommandSignatureGenerators {
    CommandSignatureGenerators::new("sysctl").add_generator(
        "completions",
        Generator::script(
            CommandBuilder::single_command("sysctl -A -N"),
            output_parsers::desc_variable,
        ),
    )
}
