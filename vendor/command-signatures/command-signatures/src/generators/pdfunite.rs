use super::template_filters;
use brasa_completion_metadata::CommandSignatureGenerators;

pub fn generator() -> CommandSignatureGenerators {
    CommandSignatureGenerators::new("pdfunite").add_filter("filter-pdf", template_filters::pdf())
}
