use crate::error::UserFacingError;
use crate::schema;

#[derive(cynic::QueryVariables, Debug)]
pub struct ListBrasaDevImagesVariables {}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "RootQuery", variables = "ListBrasaDevImagesVariables")]
pub struct ListBrasaDevImages {
    #[cynic(rename = "listBrasaDevImages")]
    pub list_brasa_dev_images: ListBrasaDevImagesResult,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ListBrasaDevImagesOutput {
    pub images: Vec<ImageTag>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ImageTag {
    pub image: String,
    pub repository: String,
    pub tag: String,
}

#[derive(cynic::InlineFragments, Debug)]
pub enum ListBrasaDevImagesResult {
    ListBrasaDevImagesOutput(ListBrasaDevImagesOutput),
    UserFacingError(UserFacingError),
    #[cynic(fallback)]
    Unknown,
}

crate::client::define_operation! {
    ListBrasaDevImages(ListBrasaDevImagesVariables) -> ListBrasaDevImages;
}
