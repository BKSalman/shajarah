use uuid::Uuid;

pub mod add_member_modal;
pub mod add_requests_grid;
pub mod edit_member_modal;
pub mod family_management_header;
pub mod member_card;
pub mod members_grid;
pub mod request_card;
pub mod view_member_modal;
pub mod view_request_modal;

#[derive(PartialEq, Clone)]
enum ImageState {
    Loading,
    UserImage,
    GeneratedAvatar,
    DefaultIcon,
    Error,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ShowModal {
    ViewMember(i64),
    ViewRequest(Uuid),
    AddMember,
    EditMember(i64),
    DeleteMember(i64),
    UserInvite,
}
