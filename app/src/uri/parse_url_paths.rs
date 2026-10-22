use url::Url;

use crate::ChannelState;
use crate::cloud_object::extract_server_id_and_object_type_from_brasa_drive_link;
use crate::drive::OpenBrasaDriveObjectArgs;

#[derive(PartialEq, Debug)]
pub enum BrasaWebLink {
    Session,
    DriveObject(Box<OpenBrasaDriveObjectArgs>),
}

pub fn get_item_data_from_brasa_link(url: &Url) -> Option<BrasaWebLink> {
    if url.origin() == ChannelState::server_root_domain() {
        url.path_segments().and_then(|mut path_segments| {
            path_segments.next().and_then(|segment| match segment {
                "drive" => extract_server_id_and_object_type_from_brasa_drive_link(url)
                    .map(|args| BrasaWebLink::DriveObject(Box::new(args))),
                "session" => Some(BrasaWebLink::Session),
                _ => None,
            })
        })
    } else {
        None
    }
}
