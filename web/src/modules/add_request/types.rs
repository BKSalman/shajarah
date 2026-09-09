use dioxus::prelude::*;
use indexmap::IndexMap;
use jiff::Zoned;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::i18n::Arabic;
use crate::modules::member::types::{Gender, MarriageStatus};
use crate::modules::settings::types::RequestRules;

/// Fields whose requiredness the admin controls are validated with
/// `custom` rules instead of `required`, because `#[garde(...)]` is resolved at
/// compile time while [`RequestRules`] is only known at runtime. Both structs
/// must name the same context type — `#[garde(dive)]` propagates it into
/// `children`.
#[derive(Debug, Clone, Store, garde::Validate, Serialize, Deserialize)]
#[garde(context(RequestRules as rules))]
pub struct RequestData {
    #[garde(required, length(min = 1))]
    pub name: Option<String>,
    #[garde(required, length(min = 1))]
    pub last_name: Option<String>,
    #[garde(required)]
    pub gender: Option<Gender>,
    #[garde(custom(require_birthday))]
    pub birthday: Option<Zoned>,
    #[garde(required)]
    pub father_id: Option<i64>,
    #[garde(skip)]
    pub mother_id: Option<i64>,
    #[garde(custom(require_info_keys))]
    pub info: IndexMap<String, String>,
    #[garde(custom(require_image(self.gender)))]
    pub image: Option<Vec<u8>>,
    #[garde(skip)]
    pub image_type: Option<String>,
    #[garde(dive)]
    pub spouse: Option<RequestSpouseData>,
    #[garde(dive)]
    pub children: Vec<RequestChildData>,
}

/// Which of the two ways a spouse was given: an existing member of the tree, or
/// somebody from outside it who gets created when the request is approved.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpouseKind {
    #[default]
    Existing,
    New,
}

/// The fields for [`SpouseKind::New`] mirror [`RequestChildData`], with one
/// addition: an outside spouse keeps their own family name instead of
/// inheriting the requested member's.
#[derive(Default, Debug, Clone, Store, garde::Validate, Serialize, Deserialize)]
#[garde(context(RequestRules as rules))]
pub struct RequestSpouseData {
    #[garde(skip)]
    pub kind: SpouseKind,
    #[garde(custom(require_picked_member(self.kind)))]
    pub member_id: Option<i64>,
    #[garde(custom(require_new_spouse_text(self.kind)))]
    pub name: Option<String>,
    #[garde(custom(require_new_spouse_text(self.kind)))]
    pub last_name: Option<String>,
    #[garde(custom(require_new_spouse_gender(self.kind)))]
    pub gender: Option<Gender>,
    #[garde(custom(require_new_spouse_birthday(self.kind)))]
    pub birthday: Option<Zoned>,
    #[garde(custom(require_new_spouse_info(self.kind)))]
    pub info: IndexMap<String, String>,
    #[garde(custom(require_new_spouse_image(self.kind, self.gender)))]
    pub image: Option<Vec<u8>>,
    #[garde(skip)]
    pub image_type: Option<String>,
    #[garde(required)]
    pub status: Option<MarriageStatus>,
}

#[derive(Default, Debug, Clone, Store, garde::Validate, Serialize, Deserialize)]
#[garde(context(RequestRules as rules))]
pub struct RequestChildData {
    #[garde(required, length(min = 1))]
    pub name: Option<String>,
    #[garde(required)]
    pub gender: Option<Gender>,
    #[garde(custom(require_birthday))]
    pub birthday: Option<Zoned>,
    #[garde(custom(require_info_keys))]
    pub info: IndexMap<String, String>,
    #[garde(custom(require_image(self.gender)))]
    pub image: Option<Vec<u8>>,
    #[garde(skip)]
    pub image_type: Option<String>,
    /// The child belongs to the couple, not just to the requested member.
    /// Ignored when the request carries no spouse.
    #[garde(skip)]
    pub from_spouse: bool,
}

pub fn missing_required_info(info: &IndexMap<String, String>, keys: &[String]) -> Vec<String> {
    keys.iter()
        .filter(|key| info.get(*key).is_none_or(|value| value.trim().is_empty()))
        .cloned()
        .collect()
}

/// The same message `#[garde(required)]` produces under `with_i18n(Arabic, ..)`.
///
/// Custom validators can't reach the installed i18n handler — garde keeps it
/// `pub(crate)` — so we call our own impl directly. As a bonus these messages
/// come out Arabic on the server too, which doesn't wrap validation in `with_i18n`.
fn required_message() -> garde::Error {
    use garde::I18n;

    garde::Error::new(Arabic.required_not_set())
}

fn require_birthday(value: &Option<Zoned>, rules: &RequestRules) -> garde::Result {
    if rules.require_birthday && value.is_none() {
        return Err(required_message());
    }

    Ok(())
}

fn require_image(
    gender: Option<Gender>,
) -> impl FnOnce(&Option<Vec<u8>>, &RequestRules) -> garde::Result {
    move |value, rules| {
        if rules.require_image && value.is_none() && gender != Some(Gender::Female) {
            return Err(required_message());
        }

        Ok(())
    }
}

pub fn image_required_for(gender: Option<Gender>, rules: &RequestRules) -> bool {
    rules.require_image && gender != Some(Gender::Female)
}

fn require_picked_member(
    kind: SpouseKind,
) -> impl FnOnce(&Option<i64>, &RequestRules) -> garde::Result {
    move |value, _| {
        if kind == SpouseKind::Existing && value.is_none() {
            return Err(required_message());
        }

        Ok(())
    }
}

fn require_new_spouse_text(
    kind: SpouseKind,
) -> impl FnOnce(&Option<String>, &RequestRules) -> garde::Result {
    move |value, _| {
        if kind == SpouseKind::New && value.as_ref().is_none_or(|v| v.trim().is_empty()) {
            return Err(required_message());
        }

        Ok(())
    }
}

fn require_new_spouse_gender(
    kind: SpouseKind,
) -> impl FnOnce(&Option<Gender>, &RequestRules) -> garde::Result {
    move |value, _| {
        if kind == SpouseKind::New && value.is_none() {
            return Err(required_message());
        }

        Ok(())
    }
}

fn require_new_spouse_birthday(
    kind: SpouseKind,
) -> impl FnOnce(&Option<Zoned>, &RequestRules) -> garde::Result {
    move |value, rules| {
        if kind == SpouseKind::New {
            return require_birthday(value, rules);
        }

        Ok(())
    }
}

fn require_new_spouse_info(
    kind: SpouseKind,
) -> impl FnOnce(&IndexMap<String, String>, &RequestRules) -> garde::Result {
    move |value, rules| {
        if kind == SpouseKind::New {
            return require_info_keys(value, rules);
        }

        Ok(())
    }
}

fn require_new_spouse_image(
    kind: SpouseKind,
    gender: Option<Gender>,
) -> impl FnOnce(&Option<Vec<u8>>, &RequestRules) -> garde::Result {
    move |value, rules| {
        if kind == SpouseKind::New {
            return require_image(gender)(value, rules);
        }

        Ok(())
    }
}

fn require_info_keys(value: &IndexMap<String, String>, rules: &RequestRules) -> garde::Result {
    let missing = missing_required_info(value, &rules.required_info_keys);

    if missing.is_empty() {
        return Ok(());
    }

    Err(garde::Error::new(format!(
        "الحقول التالية مطلوبة: {}",
        missing.join("، ")
    )))
}

#[cfg_attr(feature = "server", derive(sqlx::Type))]
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[cfg_attr(
    feature = "server",
    sqlx(type_name = "request_status", rename_all = "snake_case")
)]
#[serde(rename_all = "snake_case")]
pub enum RequestStatus {
    #[default]
    Pending,
    Approved,
    Disapproved,
}

#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RequestedMember {
    pub id: Uuid,
    pub name: String,
    pub gender: Gender,
    pub birthday: Option<Zoned>,
    pub last_name: String,
    pub mother_request_id: Option<Uuid>,
    pub father_request_id: Option<Uuid>,
    pub father_id: Option<i64>,
    pub father_name: Option<String>,
    pub mother_id: Option<i64>,
    pub mother_name: Option<String>,
    pub personal_info: Option<IndexMap<String, String>>,
    pub image: Option<Vec<u8>>,
    pub image_type: Option<String>,
    pub status: RequestStatus,
    pub spouse_id: Option<i64>,
    pub spouse_of_request_id: Option<Uuid>,
    pub spouse_name: Option<String>,
    pub marriage_status: Option<MarriageStatus>,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct RequestedMemberBrief {
    pub id: Uuid,
    pub name: String,
    pub gender: Gender,
    pub birthday: Option<jiff_sqlx::Timestamp>,
    pub last_name: String,
    pub father_id: Option<i64>,
    pub mother_id: Option<i64>,
    pub personal_info: Option<sqlx::types::Json<IndexMap<String, String>>>,
    pub image: Option<Vec<u8>>,
    pub image_type: Option<String>,
    pub status: RequestStatus,
    pub mother_request_id: Option<Uuid>,
    pub father_request_id: Option<Uuid>,
    pub spouse_id: Option<i64>,
    pub spouse_of_request_id: Option<Uuid>,
    pub marriage_status: Option<MarriageStatus>,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct RequestedMemberRowWithParents {
    pub id: Uuid,
    pub name: String,
    pub gender: Gender,
    pub birthday: Option<jiff_sqlx::Timestamp>,
    pub last_name: String,
    pub image: Option<Vec<u8>>,
    pub image_type: Option<String>,
    pub mother_id: Option<i64>,
    pub father_id: Option<i64>,
    pub personal_info: Option<serde_json::Value>,
    pub mother_request_id: Option<Uuid>,
    pub father_request_id: Option<Uuid>,
    pub mother_name: Option<String>,
    pub mother_gender: Option<Gender>,
    pub mother_birthday: Option<jiff_sqlx::Timestamp>,
    pub mother_last_name: Option<String>,
    pub father_name: Option<String>,
    pub father_gender: Option<Gender>,
    pub father_birthday: Option<jiff_sqlx::Timestamp>,
    pub father_last_name: Option<String>,
    pub status: RequestStatus,
    pub spouse_id: Option<i64>,
    pub spouse_of_request_id: Option<Uuid>,
    pub spouse_name: Option<String>,
    pub marriage_status: Option<MarriageStatus>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use garde::Validate;

    fn valid_request() -> RequestData {
        RequestData {
            name: Some("سلمان".to_string()),
            last_name: Some("أبوحيمد".to_string()),
            gender: Some(Gender::Male),
            birthday: None,
            father_id: Some(1),
            mother_id: None,
            info: IndexMap::new(),
            image: None,
            image_type: None,
            spouse: None,
            children: Vec::new(),
        }
    }

    fn picked_spouse() -> RequestSpouseData {
        RequestSpouseData {
            kind: SpouseKind::Existing,
            member_id: Some(42),
            status: Some(MarriageStatus::Married),
            ..RequestSpouseData::default()
        }
    }

    fn error_paths(report: &garde::Report) -> Vec<String> {
        report.iter().map(|(path, _)| path.to_string()).collect()
    }

    #[test]
    fn birthday_is_required_only_when_the_admin_says_so() {
        let request = valid_request();

        let relaxed = RequestRules {
            require_birthday: false,
            ..RequestRules::default()
        };
        assert!(request.validate_with(&relaxed).is_ok());

        let report = request
            .validate_with(&RequestRules::default())
            .expect_err("birthday defaults to required");
        assert_eq!(error_paths(&report), ["birthday"]);
    }

    #[test]
    fn image_is_optional_by_default() {
        let request = valid_request();

        let rules = RequestRules {
            require_birthday: false,
            require_image: true,
            ..RequestRules::default()
        };

        assert!(
            request
                .validate_with(&RequestRules {
                    require_birthday: false,
                    ..RequestRules::default()
                })
                .is_ok()
        );

        let report = request
            .validate_with(&rules)
            .expect_err("image is required");
        assert_eq!(error_paths(&report), ["image"]);
    }

    #[test]
    fn females_are_exempt_from_the_image_requirement() {
        let rules = RequestRules {
            require_birthday: false,
            require_image: true,
            ..RequestRules::default()
        };

        let mut request = valid_request();
        request.gender = Some(Gender::Female);
        request.children.push(RequestChildData {
            name: Some("نورة".to_string()),
            gender: Some(Gender::Female),
            ..RequestChildData::default()
        });

        assert!(request.validate_with(&rules).is_ok());

        // A son in the same request still needs one.
        request.children.push(RequestChildData {
            name: Some("عبدالله".to_string()),
            gender: Some(Gender::Male),
            ..RequestChildData::default()
        });

        let report = request
            .validate_with(&rules)
            .expect_err("the son still needs an image");
        assert_eq!(error_paths(&report), ["children[1].image"]);
    }

    #[test]
    fn a_picked_spouse_only_needs_a_member_and_a_state() {
        let rules = RequestRules {
            require_birthday: false,
            require_image: true,
            required_info_keys: vec!["المهنة".to_string()],
            ..RequestRules::default()
        };

        let mut request = valid_request();
        request
            .info
            .insert("المهنة".to_string(), "محامي".to_string());
        request.image = Some(vec![0]);
        request.spouse = Some(picked_spouse());

        // None of the admin's field rules apply to somebody already in the tree.
        assert!(request.validate_with(&rules).is_ok());

        request.spouse = Some(RequestSpouseData {
            member_id: None,
            ..picked_spouse()
        });
        let report = request
            .validate_with(&rules)
            .expect_err("a picked spouse needs a member");
        assert_eq!(error_paths(&report), ["spouse.member_id"]);
    }

    #[test]
    fn a_new_spouse_reports_the_paths_the_form_uses() {
        let rules = RequestRules {
            require_birthday: false,
            ..RequestRules::default()
        };

        let mut request = valid_request();
        request.spouse = Some(RequestSpouseData {
            kind: SpouseKind::New,
            ..RequestSpouseData::default()
        });

        let report = request
            .validate_with(&rules)
            .expect_err("a blank new spouse is not valid");

        let mut paths = error_paths(&report);
        paths.sort();
        assert_eq!(
            paths,
            [
                "spouse.gender",
                "spouse.last_name",
                "spouse.name",
                "spouse.status",
            ]
        );
    }

    #[test]
    fn a_new_spouse_follows_the_admins_field_rules() {
        let rules = RequestRules {
            require_birthday: false,
            require_image: true,
            ..RequestRules::default()
        };

        let mut spouse = RequestSpouseData {
            kind: SpouseKind::New,
            name: Some("نورة".to_string()),
            last_name: Some("القحطاني".to_string()),
            gender: Some(Gender::Female),
            status: Some(MarriageStatus::Married),
            ..RequestSpouseData::default()
        };

        let mut request = valid_request();
        request.image = Some(vec![0]);
        request.spouse = Some(spouse.clone());

        // the female exemption reaches an outside spouse too
        assert!(request.validate_with(&rules).is_ok());

        spouse.gender = Some(Gender::Male);
        spouse.name = Some("خالد".to_string());
        request.gender = Some(Gender::Female);
        request.spouse = Some(spouse);

        let report = request
            .validate_with(&rules)
            .expect_err("a male outside spouse still needs an image");
        assert_eq!(error_paths(&report), ["spouse.image"]);
    }

    #[test]
    fn required_info_keys_must_be_filled_in() {
        let rules = RequestRules {
            require_birthday: false,
            require_image: false,
            required_info_keys: vec!["المهنة".to_string(), "مكان الميلاد".to_string()],
        };

        // Absent entirely.
        let mut request = valid_request();
        let report = request
            .validate_with(&rules)
            .expect_err("both keys are missing");
        assert_eq!(error_paths(&report), ["info"]);

        // Present but blank counts as missing.
        request.info.insert("المهنة".to_string(), "  ".to_string());
        request
            .info
            .insert("مكان الميلاد".to_string(), String::new());
        assert_eq!(
            missing_required_info(&request.info, &rules.required_info_keys),
            ["المهنة", "مكان الميلاد"]
        );

        request
            .info
            .insert("المهنة".to_string(), "محامي".to_string());
        request
            .info
            .insert("مكان الميلاد".to_string(), "الرياض".to_string());
        assert!(request.validate_with(&rules).is_ok());
    }

    #[test]
    fn rules_reach_children_through_dive() {
        let rules = RequestRules {
            require_birthday: false,
            require_image: true,
            required_info_keys: vec!["المهنة".to_string()],
        };

        let mut request = valid_request();
        request.image = Some(vec![0]);
        request
            .info
            .insert("المهنة".to_string(), "محامي".to_string());
        request.children.push(RequestChildData {
            name: Some("عبدالله".to_string()),
            gender: Some(Gender::Male),
            ..RequestChildData::default()
        });

        let report = request
            .validate_with(&rules)
            .expect_err("the child is missing both an image and the required info key");

        // The form builds these exact keys with `format!("children[{idx}].{field}")`.
        let mut paths = error_paths(&report);
        paths.sort();
        assert_eq!(paths, ["children[0].image", "children[0].info"]);
    }
}
