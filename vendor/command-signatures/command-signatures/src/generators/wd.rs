use super::output_parsers;
use brasa_completion_metadata::{CommandBuilder, CommandSignatureGenerators, Generator};

pub fn generator() -> CommandSignatureGenerators {
    CommandSignatureGenerators::new("wd").add_generator(
        "cat_brasarc",
        Generator::script(
            CommandBuilder::single_command("cat ~/.brasarc"),
            output_parsers::desc_brasa_point,
        ),
    )
}
