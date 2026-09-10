use dioxus::{fullstack::FileStream, prelude::*};
use jiff::Zoned;

use crate::modules::member::{
    components::member_picker::MemberPicker,
    server::{add_marriage, add_member, edit_member_image},
    types::{Gender, MarriageStatus, MemberResponseFlat},
};

/// A spouse from outside the family: everything needed to create them as a
/// member with no parents.
#[derive(Default, Clone, PartialEq, Store)]
pub struct OutsideSpouse {
    pub name: String,
    pub last_name: String,
    pub gender: Option<Gender>,
    pub birthday: Option<Zoned>,
}

/// What the spouse controls are holding: either a member picked from the tree
/// or a new person, plus the state of the marriage.
#[derive(Clone, PartialEq, Store)]
pub struct SpouseDraft {
    pub from_outside: bool,
    pub member_id: Option<i64>,
    pub outside: OutsideSpouse,
    pub status: MarriageStatus,
}

impl Default for SpouseDraft {
    fn default() -> Self {
        Self {
            from_outside: false,
            member_id: None,
            outside: OutsideSpouse::default(),
            status: MarriageStatus::Married,
        }
    }
}

/// A spouse the caller is ready to act on.
pub enum SpouseSubmission {
    Existing {
        spouse_id: i64,
        status: MarriageStatus,
    },
    Outside {
        spouse: OutsideSpouse,
        image: Option<FileStream>,
        status: MarriageStatus,
    },
}

impl SpouseDraft {
    /// The spouse to marry, or `None` while the controls are still empty —
    /// which is how "this member has no spouse" is expressed.
    pub fn submission(&self, image: Option<FileStream>) -> Option<SpouseSubmission> {
        if self.from_outside {
            let outside = &self.outside;

            if outside.name.trim().is_empty()
                || outside.last_name.trim().is_empty()
                || outside.gender.is_none()
            {
                return None;
            }

            return Some(SpouseSubmission::Outside {
                spouse: outside.clone(),
                image,
                status: self.status,
            });
        }

        self.member_id.map(|spouse_id| SpouseSubmission::Existing {
            spouse_id,
            status: self.status,
        })
    }
}

/// Marries `member_id` to the submitted spouse, creating them first when they
/// come from outside the family. Shared by the add and edit flows.
pub async fn apply_spouse(member_id: i64, submission: SpouseSubmission) -> anyhow::Result<()> {
    let (spouse_id, status) = match submission {
        SpouseSubmission::Existing { spouse_id, status } => (spouse_id, status),
        SpouseSubmission::Outside {
            spouse,
            image,
            status,
        } => {
            let Some(gender) = spouse.gender else {
                anyhow::bail!("Gender must be set");
            };

            let spouse_id = add_member(
                spouse.name,
                spouse.last_name,
                None,
                None,
                gender,
                spouse.birthday,
            )
            .await?;

            if let Some(image) = image {
                edit_member_image(spouse_id, image).await?;
            }

            (spouse_id, status)
        }
    };

    add_marriage(member_id, spouse_id, status).await?;

    Ok(())
}

/// The controls for choosing a spouse: from the tree, or a new person from
/// outside it. The caller owns the draft and decides when to act on it.
#[component]
pub fn SpouseFields(
    draft: Store<SpouseDraft>,
    image: Signal<Option<FileStream>>,
    members: Vec<MemberResponseFlat>,
    #[props(default)] exclude_id: Option<i64>,
    /// The member being married; their spouse must be of the opposite gender.
    member_gender: Option<Gender>,
    #[props(default)] disabled: bool,
) -> Element {
    let from_outside = draft.from_outside()();

    rsx! {
        div { class: "space-y-3",
            div { class: "flex gap-4",
                label { class: "flex items-center gap-2",
                    input {
                        r#type: "radio",
                        name: "spouse_source",
                        checked: !from_outside,
                        disabled,
                        onchange: move |_| draft.from_outside().set(false),
                    }
                    "من العائلة"
                }
                label { class: "flex items-center gap-2",
                    input {
                        r#type: "radio",
                        name: "spouse_source",
                        checked: from_outside,
                        disabled,
                        onchange: move |_| draft.from_outside().set(true),
                    }
                    "من خارج العائلة"
                }
            }

            if from_outside {
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-3",
                    input {
                        r#type: "text",
                        class: "input w-full",
                        placeholder: "الاسم الأول",
                        disabled,
                        value: "{draft.outside().name()}",
                        oninput: move |evt| draft.outside().name().set(evt.value()),
                    }
                    input {
                        r#type: "text",
                        class: "input w-full",
                        placeholder: "اسم العائلة",
                        disabled,
                        value: "{draft.outside().last_name()}",
                        oninput: move |evt| draft.outside().last_name().set(evt.value()),
                    }
                    select {
                        class: "dropdown w-full",
                        disabled,
                        onchange: move |evt| {
                            draft.outside().gender().set(evt.value().parse::<Gender>().ok());
                        },
                        option {
                            value: "",
                            selected: draft.outside().gender()().is_none(),
                            "اختر الجنس"
                        }
                        option {
                            value: "male",
                            selected: draft.outside().gender()() == Some(Gender::Male),
                            "ذكر"
                        }
                        option {
                            value: "female",
                            selected: draft.outside().gender()() == Some(Gender::Female),
                            "أنثى"
                        }
                    }
                    input {
                        r#type: "date",
                        class: "input w-full",
                        disabled,
                        onchange: move |evt| {
                            draft
                                .outside()
                                .birthday()
                                .set(
                                    evt
                                        .value()
                                        .parse::<jiff::civil::Date>()
                                        .ok()
                                        .and_then(|d| d.to_zoned(jiff::tz::TimeZone::UTC).ok()),
                                );
                        },
                    }
                    input {
                        r#type: "file",
                        accept: "image/*",
                        class: "input w-full md:col-span-2",
                        disabled,
                        oninput: move |evt| {
                            if let Some(file) = evt.files().into_iter().next() {
                                image.set(Some(file.into()));
                            }
                        },
                    }
                }
            } else if let Some(gender) = member_gender {
                MemberPicker {
                    members: members.clone().into_iter().map(|m| m.into()).collect(),
                    exclude_id,
                    required_gender: Some(gender.opposite()),
                    placeholder: "ابحث عن الزوج/الزوجة بالاسم...".to_string(),
                    on_select: move |id: Option<i64>| draft.member_id().set(id),
                }
            } else {
                p { class: "text-sm text-gray-500", "اختر جنس العضو أولاً" }
            }

            select {
                class: "dropdown",
                disabled,
                onchange: move |evt| {
                    if let Ok(status) = evt.value().parse::<MarriageStatus>() {
                        draft.status().set(status);
                    }
                },
                option {
                    value: "married",
                    selected: draft.status()() == MarriageStatus::Married,
                    "متزوج"
                }
                option {
                    value: "separated",
                    selected: draft.status()() == MarriageStatus::Separated,
                    "منفصل"
                }
            }
        }
    }
}
