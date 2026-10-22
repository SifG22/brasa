use super::template_filters;
use brasa_completion_metadata::CommandSignatureGenerators;

pub fn generator() -> CommandSignatureGenerators {
    CommandSignatureGenerators::new("ansible-playbook")
        .add_filter("filter-yml-yaml", template_filters::yml_yaml())
}
