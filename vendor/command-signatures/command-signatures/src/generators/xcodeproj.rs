use super::template_filters;
use brasa_completion_metadata::CommandSignatureGenerators;

pub fn generator() -> CommandSignatureGenerators {
    CommandSignatureGenerators::new("xcodeproj")
        .add_filter("filter-xcodeproj", template_filters::xcodeproj())
}
