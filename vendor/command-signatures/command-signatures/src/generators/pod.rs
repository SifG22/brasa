use super::template_filters;
use brasa_completion_metadata::CommandSignatureGenerators;

pub fn generator() -> CommandSignatureGenerators {
    CommandSignatureGenerators::new("pod").add_filter("filter-podspec", template_filters::podspec())
}
