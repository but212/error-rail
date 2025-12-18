use criterion::Criterion;
use error_rail::traits::TransientError;
use std::sync::OnceLock;
use std::time::Duration;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ============================================================================
// Test Data & Domain Types
// ============================================================================

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UserData {
    pub user_id: u64,
    pub username: String,
    pub email: String,
    pub metadata: std::collections::HashMap<String, String>,
}

impl UserData {
    pub fn new(id: u64) -> Self {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("role".to_string(), "user".to_string());
        metadata.insert("department".to_string(), "engineering".to_string());
        metadata.insert("location".to_string(), "seoul".to_string());

        Self {
            user_id: id,
            username: format!("user_{id}"),
            email: format!("user{id}@company.com"),
            metadata,
        }
    }
}

pub fn realistic_user_data() -> &'static Vec<UserData> {
    static INSTANCE: OnceLock<Vec<UserData>> = OnceLock::new();
    INSTANCE.get_or_init(|| (0..1000).map(UserData::new).collect())
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub enum DomainError {
    Database(String),
    Network(String),
    Validation(String),
    Authentication(String),
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainError::Database(msg) => write!(f, "Database error: {msg}"),
            DomainError::Network(msg) => write!(f, "Network error: {msg}"),
            DomainError::Validation(msg) => write!(f, "Validation error: {msg}"),
            DomainError::Authentication(msg) => write!(f, "Authentication error: {msg}"),
        }
    }
}

impl TransientError for DomainError {
    fn is_transient(&self) -> bool {
        matches!(self, DomainError::Network(_) | DomainError::Database(_))
    }

    fn retry_after_hint(&self) -> Option<Duration> {
        match self {
            DomainError::Network(_) => Some(Duration::from_secs(1)),
            DomainError::Database(_) => Some(Duration::from_millis(100)),
            _ => None,
        }
    }

    fn max_retries_hint(&self) -> Option<u32> {
        match self {
            DomainError::Network(_) => Some(3),
            DomainError::Database(_) => Some(5),
            _ => None,
        }
    }
}

// ============================================================================
// Simulation Functions
// ============================================================================

pub fn simulate_db_query(user_id: u64) -> Result<UserData, DomainError> {
    if user_id % 100 == 0 {
        Err(DomainError::Database("Connection timeout".to_string()))
    } else {
        Ok(UserData::new(user_id))
    }
}

pub fn simulate_validation(user: UserData) -> Result<UserData, DomainError> {
    if user.user_id % 50 == 0 {
        Err(DomainError::Validation("Invalid email format".to_string()))
    } else {
        Ok(user)
    }
}

pub fn simulate_auth_check(user: UserData) -> Result<UserData, DomainError> {
    if user.user_id % 25 == 0 {
        Err(DomainError::Authentication("Token expired".to_string()))
    } else {
        Ok(user)
    }
}

pub fn validate_user_email(email: &str) -> Result<String, DomainError> {
    if email.contains('@') {
        Ok(email.to_string())
    } else {
        Err(DomainError::Validation("Invalid email format".to_string()))
    }
}

pub fn validate_user_age(age: i32) -> Result<i32, DomainError> {
    if age >= 18 && age <= 120 {
        Ok(age)
    } else {
        Err(DomainError::Validation(
            "Age out of valid range".to_string(),
        ))
    }
}

pub fn validate_user_name(name: &str) -> Result<String, DomainError> {
    if name.len() >= 2 && name.len() <= 50 {
        Ok(name.to_string())
    } else {
        Err(DomainError::Validation("Name length invalid".to_string()))
    }
}

// ============================================================================
// Criterion Configuration
// ============================================================================

pub fn configure_criterion() -> Criterion {
    Criterion::default()
        .sample_size(100)
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(5))
        .noise_threshold(0.05)
}
