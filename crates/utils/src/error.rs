use serde::{Deserialize, Serialize};
use std::{
  fmt,
  fmt::{Debug, Display},
};
use tracing_error::SpanTrace;
#[cfg(feature = "full")]
use ts_rs::TS;

pub type LemmyResult<T> = Result<T, LemmyError>;

pub struct LemmyError {
  pub error_type: Option<LemmyErrorType>,
  pub inner: anyhow::Error,
  pub context: SpanTrace,
}

impl LemmyError {
  /// Create a LemmyError from error and message, including stack trace
  pub fn from_error_and_type<E>(error: E, error_type: LemmyErrorType) -> Self
  where
    E: Into<anyhow::Error>,
  {
    LemmyError {
      error_type: Some(error_type),
      inner: error.into(),
      context: SpanTrace::capture(),
    }
  }

  /// Add message to existing LemmyError (or overwrite existing error)
  pub fn with_type(self, error_type: LemmyErrorType) -> Self {
    LemmyError {
      error_type: Some(error_type),
      ..self
    }
  }
}

impl<T> From<T> for LemmyError
where
  T: Into<anyhow::Error>,
{
  fn from(t: T) -> Self {
    LemmyError {
      error_type: None,
      inner: t.into(),
      context: SpanTrace::capture(),
    }
  }
}

impl Debug for LemmyError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("LemmyError")
      .field("message", &self.error_type)
      .field("inner", &self.inner)
      .field("context", &self.context)
      .finish()
  }
}

impl Display for LemmyError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if let Some(message) = &self.error_type {
      write!(f, "{message}: ")?;
    }
    // print anyhow including trace
    // https://docs.rs/anyhow/latest/anyhow/struct.Error.html#display-representations
    // this will print the anyhow trace (only if it exists)
    // and if RUST_BACKTRACE=1, also a full backtrace
    writeln!(f, "{:?}", self.inner)?;
    fmt::Display::fmt(&self.context, f)
  }
}

impl actix_web::error::ResponseError for LemmyError {
  fn status_code(&self) -> http::StatusCode {
    match self.inner.downcast_ref::<diesel::result::Error>() {
      Some(diesel::result::Error::NotFound) => http::StatusCode::NOT_FOUND,
      _ => http::StatusCode::BAD_REQUEST,
    }
  }

  fn error_response(&self) -> actix_web::HttpResponse {
    if let Some(message) = &self.error_type {
      actix_web::HttpResponse::build(self.status_code()).json(message)
    } else {
      actix_web::HttpResponse::build(self.status_code())
        .content_type("text/plain")
        .body(self.inner.to_string())
    }
  }
}

#[derive(Display, Debug, Serialize, Deserialize, Clone, PartialEq, EnumIter)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
#[serde(tag = "error", content = "message", rename_all = "snake_case")]
// TODO: order these based on the crate they belong to (utils, federation, db, api)
pub enum LemmyErrorType {
  ReportReasonRequired,
  ReportTooLong,
  NotAModerator,
  NotAnAdmin,
  CantBlockYourself,
  CantBlockAdmin,
  CouldntUpdateUser,
  PasswordsDoNotMatch,
  PasswordIncorrect,
  EmailNotVerified,
  EmailRequired,
  CouldntUpdateComment,
  CouldntUpdatePrivateMessage,
  CannotLeaveAdmin,
  NoLinesInHtml,
  SiteMetadataPageIsNotDoctypeHtml,
  PictrsResponseError(String),
  PictrsPurgeResponseError(String),
  ImageUrlMissingPathSegments,
  ImageUrlMissingLastPathSegment,
  PictrsApiKeyNotProvided,
  NoContentTypeHeader,
  NotAnImageType,
  NotAModOrAdmin,
  NoAdmins,
  NotTopAdmin,
  NotTopMod,
  NotLoggedIn,
  SiteBan,
  Deleted,
  BannedFromCommunity,
  CouldntFindCommunity,
  PersonIsBlocked,
  DownvotesAreDisabled,
  InstanceIsPrivate,
  InvalidPassword,
  SiteDescriptionLengthOverflow,
  HoneypotFailed,
  RegistrationApplicationIsPending,
  CantEnablePrivateInstanceAndFederationTogether,
  Locked,
  CouldntCreateComment,
  MaxCommentDepthReached,
  NoCommentEditAllowed,
  OnlyAdminsCanCreateCommunities,
  CommunityAlreadyExists,
  LanguageNotAllowed,
  OnlyModsCanPostInCommunity,
  CouldntUpdatePost,
  NoPostEditAllowed,
  CouldntFindPost,
  EditPrivateMessageNotAllowed,
  SiteAlreadyExists,
  ApplicationQuestionRequired,
  InvalidDefaultPostListingType,
  RegistrationClosed,
  RegistrationApplicationAnswerRequired,
  EmailAlreadyExists,
  FederationForbiddenByStrictAllowList,
  PersonIsBannedFromCommunity,
  ObjectIsNotPublic,
  InvalidCommunity,
  CannotCreatePostOrCommentInDeletedOrRemovedCommunity,
  CannotReceivePage,
  NewPostCannotBeLocked,
  OnlyLocalAdminCanRemoveCommunity,
  OnlyLocalAdminCanRestoreCommunity,
  NoIdGiven,
  CouldntFindUsernameOrEmail,
  InvalidQuery,
  ObjectNotLocal,
  PostIsLocked,
  PersonIsBannedFromSite,
  InvalidVoteValue,
  PageDoesNotSpecifyCreator,
  PageDoesNotSpecifyGroup,
  NoCommunityFoundInCc,
  NoEmailSetup,
  EmailSmtpServerNeedsAPort,
  MissingAnEmail,
  RateLimitError,
  InvalidName,
  InvalidDisplayName,
  InvalidMatrixId,
  InvalidPostTitle,
  InvalidBodyField,
  BioLengthOverflow,
  MissingTotpToken,
  IncorrectTotpToken,
  CouldntParseTotpSecret,
  CouldntLikeComment,
  CouldntSaveComment,
  CouldntCreateReport,
  CouldntResolveReport,
  CommunityModeratorAlreadyExists,
  CommunityUserAlreadyBanned,
  CommunityBlockAlreadyExists,
  CommunityFollowerAlreadyExists,
  CouldntUpdateCommunityHiddenStatus,
  PersonBlockAlreadyExists,
  UserAlreadyExists,
  TokenNotFound,
  CouldntLikePost,
  CouldntSavePost,
  CouldntMarkPostAsRead,
  CouldntUpdateCommunity,
  CouldntUpdateReplies,
  CouldntUpdatePersonMentions,
  PostTitleTooLong,
  CouldntCreatePost,
  CouldntCreatePrivateMessage,
  CouldntUpdatePrivate,
  SystemErrLogin,
  CouldntSetAllRegistrationsAccepted,
  CouldntSetAllEmailVerified,
  Banned,
  CouldntGetComments,
  CouldntGetPosts,
  InvalidUrl,
  EmailSendFailed,
  Slurs,
  CouldntGenerateTotp,
  CouldntFindObject,
  RegistrationDenied(String),
  FederationDisabled,
  DomainBlocked,
  DomainNotInAllowList,
  FederationDisabledByStrictAllowList,
  SiteNameRequired,
  SiteNameLengthOverflow,
  PermissiveRegex,
  InvalidRegex,
  CaptchaIncorrect,
  PasswordResetLimitReached,
  CouldntCreateAudioCaptcha,
  Unknown,
}

impl From<LemmyErrorType> for LemmyError {
  fn from(error_type: LemmyErrorType) -> Self {
    let inner = anyhow::anyhow!("{}", error_type);
    LemmyError {
      error_type: Some(error_type),
      inner,
      context: SpanTrace::capture(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use actix_web::{body::MessageBody, ResponseError};
  use std::fs::read_to_string;
  use strum::IntoEnumIterator;

  #[test]
  fn deserializes_no_message() {
    let err = LemmyError::from(LemmyErrorType::Banned).error_response();
    let json = String::from_utf8(err.into_body().try_into_bytes().unwrap().to_vec()).unwrap();
    assert_eq!(&json, "{\"error\":\"banned\"}")
  }

  #[test]
  fn deserializes_with_message() {
    let reg_denied = LemmyErrorType::RegistrationDenied(String::from("reason"));
    let err = LemmyError::from(reg_denied).error_response();
    let json = String::from_utf8(err.into_body().try_into_bytes().unwrap().to_vec()).unwrap();
    assert_eq!(
      &json,
      "{\"error\":\"registration_denied\",\"message\":\"reason\"}"
    )
  }

  /// Check if errors match translations. Disabled because many are not translated at all.
  #[test]
  #[ignore]
  fn test_translations_match() {
    #[derive(Deserialize)]
    struct Err {
      error: String,
    }

    let translations = read_to_string("translations/translations/en.json").unwrap();
    LemmyErrorType::iter().for_each(|e| {
      let msg = serde_json::to_string(&e).unwrap();
      let msg: Err = serde_json::from_str(&msg).unwrap();
      let msg = msg.error;
      assert!(translations.contains(&format!("\"{msg}\"")), "{msg}");
    });
  }
}
