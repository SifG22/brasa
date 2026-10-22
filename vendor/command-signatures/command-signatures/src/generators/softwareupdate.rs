use super::output_parsers;
use brasa_completion_metadata::{CommandBuilder, CommandSignatureGenerators, Generator};

pub fn generator() -> CommandSignatureGenerators {
    CommandSignatureGenerators::new("softwareupdate").add_generator(
        "completions",
        Generator::script(
            CommandBuilder::single_command("softwareupdate --list"),
            output_parsers::softwareupdate_labels,
        ),
    )
}
