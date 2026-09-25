//! Authentication flow interface

use reqwest::StatusCode;

use crate::State;
use crate::state::{Credentials, MinecraftLoginFlow};
use crate::util::fetch::INSECURE_REQWEST_CLIENT;

#[tracing::instrument]
pub async fn check_reachable() -> crate::Result<()> {
    let resp = INSECURE_REQWEST_CLIENT
        .get("https://sessionserver.mojang.com/session/minecraft/hasJoined")
        .send()
        .await?;
    if resp.status() == StatusCode::NO_CONTENT {
        return Ok(());
    }
    resp.error_for_status()?;
    Ok(())
}

#[tracing::instrument]
pub async fn begin_login() -> crate::Result<MinecraftLoginFlow> {
    let state = State::get().await?;

    crate::state::login_begin(&state.pool).await
}

#[tracing::instrument]
pub async fn finish_login(
    code: &str,
    flow: MinecraftLoginFlow,
) -> crate::Result<Credentials> {
    let state = State::get().await?;

    let credentials =
        crate::state::login_finish(code, flow, &state.pool).await?;

    if let Err(error) =
        crate::onboarding_checklist::mark_logged_into_minecraft().await
    {
        tracing::warn!(
            "Failed to mark Minecraft login in onboarding checklist: {error}"
        );
    }

    Ok(credentials)
}

pub fn validate_offline_username(username: &str) -> crate::Result<()> {
    if !(3..=16).contains(&username.len()) {
        return Err(crate::ErrorKind::InputError(
            "Username must be between 3 and 16 characters".to_string(),
        )
        .into());
    }

    if !username
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return Err(crate::ErrorKind::InputError(
            "Username can only contain ASCII letters, numbers, and underscores"
                .to_string(),
        )
        .into());
    }

    Ok(())
}

#[tracing::instrument]
pub async fn create_offline_credentials(
    username: String,
) -> crate::Result<Credentials> {
    validate_offline_username(&username)?;
    let state = State::get().await?;
    let user_id = uuid::Uuid::new_v4();
    let credentials = Credentials {
        offline_profile: crate::state::MinecraftProfile {
            id: user_id,
            name: username,
            ..crate::state::MinecraftProfile::default()
        },
        access_token: format!("offline_token_{user_id}"),
        refresh_token: format!("offline_refresh_{user_id}"),
        expires: chrono::Utc::now() + chrono::Duration::days(36_500),
        active: true,
    };

    credentials.upsert(&state.pool).await?;
    Ok(credentials)
}

#[tracing::instrument]
pub async fn get_default_user() -> crate::Result<Option<uuid::Uuid>> {
    let state = State::get().await?;
    let user = Credentials::get_default_credential(&state.pool).await?;
    Ok(user.map(|user| user.offline_profile.id))
}

#[tracing::instrument]
pub async fn set_default_user(user: uuid::Uuid) -> crate::Result<()> {
    let state = State::get().await?;
    let users = Credentials::get_all(&state.pool).await?;
    let (_, mut user) = users.remove(&user).ok_or_else(|| {
        crate::ErrorKind::OtherError(format!(
            "Tried to get nonexistent user with ID {user}"
        ))
        .as_error()
    })?;

    user.active = true;
    user.upsert(&state.pool).await?;

    Ok(())
}

/// Remove a user account from the database
#[tracing::instrument]
pub async fn remove_user(uuid: uuid::Uuid) -> crate::Result<()> {
    let state = State::get().await?;

    let users = Credentials::get_all(&state.pool).await?;

    if let Some((uuid, user)) = users.remove(&uuid) {
        Credentials::remove(uuid, &state.pool).await?;

        if user.active
            && let Some((_, mut user)) = users.into_iter().next()
        {
            user.active = true;
            user.upsert(&state.pool).await?;
        }
    }

    Ok(())
}

/// Get a copy of the list of all user credentials
#[tracing::instrument]
pub async fn users() -> crate::Result<Vec<Credentials>> {
    let state = State::get().await?;
    let users = Credentials::get_all(&state.pool).await?;
    Ok(users.into_iter().map(|x| x.1).collect())
}

#[cfg(test)]
mod tests {
    use super::validate_offline_username;

    #[test]
    fn offline_username_validation_matches_minecraft_rules() {
        assert!(validate_offline_username("Player_123").is_ok());
        assert!(validate_offline_username("ab").is_err());
        assert!(validate_offline_username("abcdefghijklmnopq").is_err());
        assert!(validate_offline_username("bad-name").is_err());
        assert!(validate_offline_username("sp ace").is_err());
    }
}
