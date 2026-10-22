use super::template_filters;
use brasa_completion_metadata::CommandSignatureGenerators;

pub fn generator() -> CommandSignatureGenerators {
    CommandSignatureGenerators::new("wasm-bindgen")
        .add_filter("filter-wasm", template_filters::wasm())
}
