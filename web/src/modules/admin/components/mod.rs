pub mod add_member_modal;
pub mod family_management_header;
pub mod member_card;
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
