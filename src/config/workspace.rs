use crate::config::helpers::{optional_env, parse_bool_env};
use crate::error::ConfigError;
use crate::workspace::layer::MemoryLayer;

/// HDC deduplication mode.
///
/// Controls how the write-time dedup check behaves:
/// - `off`: no dedup check at all (fingerprints may still be stored in shadow mode)
/// - `warn`: similar/duplicate detected → write proceeds, warning attached to output
/// - `block`: duplicate detected → write is rejected (force=true bypasses)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum HdcDedupMode {
    /// Dedup check disabled.
    #[default]
    Off,
    /// Warn on similar/duplicate but allow the write.
    Warn,
    /// Block duplicates (force=true overrides).
    Block,
}

impl HdcDedupMode {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "off" => Some(Self::Off),
            "warn" => Some(Self::Warn),
            "block" => Some(Self::Block),
            _ => None,
        }
    }
}

/// HDC-related workspace configuration.
///
/// Controls fingerprint shadow mode, deduplication thresholds, search fusion,
/// and heartbeat novelty. Parsed from environment:
/// - `IRONCLAW_HDC_FINGERPRINT_SHADOW`: "true"/"1" to enable (default: false)
/// - `IRONCLAW_HDC_DEDUP_MODE`: "off", "warn", or "block" (default: "off")
/// - `IRONCLAW_HDC_SIMILAR_THRESHOLD`: float in (0.5, 1.0), default 0.78
/// - `IRONCLAW_HDC_DUPLICATE_THRESHOLD`: float in (similar, 1.0), default 0.92
/// - `IRONCLAW_HDC_SEARCH_SHADOW`: "true"/"1" to enable shadow mode (scores computed, no reranking)
/// - `IRONCLAW_HDC_SEARCH_LIVE`: "true"/"1" to enable live mode (scores computed AND reranking applied)
/// - `IRONCLAW_HDC_HEARTBEAT_OBSERVE`: "true"/"1" to enable (default: false)
///
/// `IRONCLAW_HDC_SEARCH_SHADOW` and `IRONCLAW_HDC_SEARCH_LIVE` are mutually exclusive.
/// Shadow mode is the recommended first-enable state.
#[derive(Debug, Clone)]
pub struct HdcConfig {
    /// Whether to compute and store fingerprints on workspace writes (shadow mode).
    pub fingerprint_shadow: bool,
    /// Dedup behavior mode.
    pub dedup_mode: HdcDedupMode,
    /// Similarity threshold for "similar" warnings.
    pub similar_threshold: f64,
    /// Similarity threshold for "duplicate" blocking.
    pub duplicate_threshold: f64,
    /// Whether HDC search fusion is in shadow mode (scores computed, no reranking).
    pub search_shadow: bool,
    /// Whether HDC search fusion is in live mode (scores computed AND reranking applied).
    pub search_live: bool,
    /// Whether heartbeat HDC observe-only mode is on.
    pub heartbeat_observe: bool,
}

impl Default for HdcConfig {
    fn default() -> Self {
        Self {
            fingerprint_shadow: false,
            dedup_mode: HdcDedupMode::Off,
            similar_threshold: 0.78,
            duplicate_threshold: 0.92,
            search_shadow: false,
            search_live: false,
            heartbeat_observe: false,
        }
    }
}

impl HdcConfig {
    /// Resolve from environment variables.
    pub fn resolve() -> Result<Self, ConfigError> {
        let dedup_mode = match optional_hdc_env("IRONCLAW_HDC_DEDUP_MODE", "HDC_DEDUP_MODE")? {
            Some((key, s)) => {
                HdcDedupMode::from_str(&s).ok_or_else(|| ConfigError::InvalidValue {
                    key: key.to_string(),
                    message: format!("must be 'off', 'warn', or 'block', got '{s}'"),
                })?
            }
            None => HdcDedupMode::Off,
        };

        let (similar_key, similar_threshold) = match optional_hdc_env(
            "IRONCLAW_HDC_SIMILAR_THRESHOLD",
            "HDC_DEDUP_SIMILAR_THRESHOLD",
        )? {
            Some((key, s)) => (
                key,
                s.parse::<f64>().map_err(|e| ConfigError::InvalidValue {
                    key: key.to_string(),
                    message: format!("must be a float: {e}"),
                })?,
            ),
            None => ("IRONCLAW_HDC_SIMILAR_THRESHOLD", 0.78),
        };

        let (duplicate_key, duplicate_threshold) = match optional_hdc_env(
            "IRONCLAW_HDC_DUPLICATE_THRESHOLD",
            "HDC_DEDUP_DUPLICATE_THRESHOLD",
        )? {
            Some((key, s)) => (
                key,
                s.parse::<f64>().map_err(|e| ConfigError::InvalidValue {
                    key: key.to_string(),
                    message: format!("must be a float: {e}"),
                })?,
            ),
            None => ("IRONCLAW_HDC_DUPLICATE_THRESHOLD", 0.92),
        };

        if !similar_threshold.is_finite() || similar_threshold <= 0.5 {
            return Err(ConfigError::InvalidValue {
                key: similar_key.to_string(),
                message: format!("must be a finite float > 0.5, got {similar_threshold}"),
            });
        }
        if !duplicate_threshold.is_finite() || duplicate_threshold >= 1.0 {
            return Err(ConfigError::InvalidValue {
                key: duplicate_key.to_string(),
                message: format!("must be a finite float < 1.0, got {duplicate_threshold}"),
            });
        }
        if similar_threshold >= duplicate_threshold {
            return Err(ConfigError::InvalidValue {
                key: similar_key.to_string(),
                message: format!(
                    "similar_threshold ({similar_threshold}) must be < duplicate_threshold ({duplicate_threshold})"
                ),
            });
        }

        let fingerprint_shadow = parse_bool_env("IRONCLAW_HDC_FINGERPRINT_SHADOW", false)?;
        let search_shadow = parse_bool_env("IRONCLAW_HDC_SEARCH_SHADOW", false)?;
        let search_live = parse_bool_env("IRONCLAW_HDC_SEARCH_LIVE", false)?;
        let heartbeat_observe = parse_bool_env("IRONCLAW_HDC_HEARTBEAT_OBSERVE", false)?;

        if search_shadow && search_live {
            return Err(ConfigError::InvalidValue {
                key: "IRONCLAW_HDC_SEARCH_LIVE".to_string(),
                message:
                    "IRONCLAW_HDC_SEARCH_SHADOW and IRONCLAW_HDC_SEARCH_LIVE are mutually exclusive"
                        .to_string(),
            });
        }

        Ok(Self {
            fingerprint_shadow,
            dedup_mode,
            similar_threshold,
            duplicate_threshold,
            search_shadow,
            search_live,
            heartbeat_observe,
        })
    }
}

fn optional_hdc_env(
    primary: &'static str,
    legacy: &'static str,
) -> Result<Option<(&'static str, String)>, ConfigError> {
    if let Some(value) = optional_env(primary)? {
        return Ok(Some((primary, value)));
    }
    optional_env(legacy).map(|value| value.map(|value| (legacy, value)))
}

#[cfg(test)]
mod hdc_config_tests {
    use super::*;
    use crate::config::helpers::lock_env;

    const HDC_KEYS: &[&str] = &[
        "IRONCLAW_HDC_DEDUP_MODE",
        "HDC_DEDUP_MODE",
        "IRONCLAW_HDC_SIMILAR_THRESHOLD",
        "HDC_DEDUP_SIMILAR_THRESHOLD",
        "IRONCLAW_HDC_DUPLICATE_THRESHOLD",
        "HDC_DEDUP_DUPLICATE_THRESHOLD",
        "IRONCLAW_HDC_FINGERPRINT_SHADOW",
        "IRONCLAW_HDC_SEARCH_SHADOW",
        "IRONCLAW_HDC_SEARCH_LIVE",
        "IRONCLAW_HDC_HEARTBEAT_OBSERVE",
    ];

    fn with_clean_hdc_env(f: impl FnOnce()) {
        let _guard = lock_env();
        let previous: Vec<(&str, Option<String>)> = HDC_KEYS
            .iter()
            .map(|key| (*key, std::env::var(key).ok()))
            .collect();
        for key in HDC_KEYS {
            unsafe { std::env::remove_var(key) };
        }
        f();
        for (key, value) in previous {
            match value {
                Some(value) => unsafe { std::env::set_var(key, value) },
                None => unsafe { std::env::remove_var(key) },
            }
        }
    }

    #[test]
    fn hdc_dedup_defaults_off() {
        with_clean_hdc_env(|| {
            let config = HdcConfig::resolve().expect("resolve hdc config");
            assert_eq!(config.dedup_mode, HdcDedupMode::Off);
            assert_eq!(config.similar_threshold, 0.78);
            assert_eq!(config.duplicate_threshold, 0.92);
        });
    }

    #[test]
    fn hdc_primary_env_names_match_example() {
        with_clean_hdc_env(|| {
            unsafe {
                std::env::set_var("IRONCLAW_HDC_DEDUP_MODE", "block");
                std::env::set_var("IRONCLAW_HDC_SIMILAR_THRESHOLD", "0.70");
                std::env::set_var("IRONCLAW_HDC_DUPLICATE_THRESHOLD", "0.95");
            }

            let config = HdcConfig::resolve().expect("resolve hdc config");
            assert_eq!(config.dedup_mode, HdcDedupMode::Block);
            assert_eq!(config.similar_threshold, 0.70);
            assert_eq!(config.duplicate_threshold, 0.95);
        });
    }

    #[test]
    fn hdc_legacy_env_names_still_work() {
        with_clean_hdc_env(|| {
            unsafe {
                std::env::set_var("HDC_DEDUP_MODE", "warn");
                std::env::set_var("HDC_DEDUP_SIMILAR_THRESHOLD", "0.76");
                std::env::set_var("HDC_DEDUP_DUPLICATE_THRESHOLD", "0.94");
            }

            let config = HdcConfig::resolve().expect("resolve hdc config");
            assert_eq!(config.dedup_mode, HdcDedupMode::Warn);
            assert_eq!(config.similar_threshold, 0.76);
            assert_eq!(config.duplicate_threshold, 0.94);
        });
    }

    #[test]
    fn hdc_primary_env_overrides_legacy() {
        with_clean_hdc_env(|| {
            unsafe {
                std::env::set_var("IRONCLAW_HDC_DEDUP_MODE", "block");
                std::env::set_var("HDC_DEDUP_MODE", "warn");
            }

            let config = HdcConfig::resolve().expect("resolve hdc config");
            assert_eq!(config.dedup_mode, HdcDedupMode::Block);
        });
    }

    #[test]
    fn hdc_rejects_non_finite_threshold() {
        with_clean_hdc_env(|| {
            unsafe {
                std::env::set_var("IRONCLAW_HDC_SIMILAR_THRESHOLD", "NaN");
            }

            let result = HdcConfig::resolve();
            assert!(result.is_err(), "NaN threshold should be rejected");
        });
    }

    #[test]
    fn hdc_rejects_invalid_bool() {
        with_clean_hdc_env(|| {
            unsafe {
                std::env::set_var("IRONCLAW_HDC_FINGERPRINT_SHADOW", "yes");
            }

            let result = HdcConfig::resolve();
            assert!(result.is_err(), "invalid boolean should be rejected");
        });
    }
}

/// Workspace-level configuration (memory layers, read scopes).
///
/// Parsed from environment variables. Lives outside of `GatewayConfig`
/// so that non-gateway channels can eventually use the same settings.
#[derive(Debug, Clone, Default)]
pub struct WorkspaceConfig {
    /// Memory layer definitions (JSON in `MEMORY_LAYERS` env var, or defaults).
    pub memory_layers: Vec<MemoryLayer>,
    /// Additional user scopes for workspace reads.
    ///
    /// When set, the workspace can read (search, read, list) from these
    /// additional user scopes while writes remain isolated to the primary
    /// `user_id`. Parsed from `WORKSPACE_READ_SCOPES` (comma-separated).
    pub read_scopes: Vec<String>,
    /// HDC deduplication configuration.
    pub hdc: HdcConfig,
}

impl WorkspaceConfig {
    /// Resolve workspace config from environment variables.
    ///
    /// `user_id` is used to derive default memory layers when `MEMORY_LAYERS`
    /// is not set.
    pub fn resolve(user_id: &str) -> Result<Self, ConfigError> {
        // --- Memory layers ---
        let memory_layers: Vec<MemoryLayer> = match optional_env("MEMORY_LAYERS")? {
            Some(json_str) => {
                serde_json::from_str(&json_str).map_err(|e| ConfigError::InvalidValue {
                    key: "MEMORY_LAYERS".to_string(),
                    message: format!("must be valid JSON array of layer objects: {e}"),
                })?
            }
            None => MemoryLayer::default_for_user(user_id),
        };

        // Validate layer names and scopes
        for layer in &memory_layers {
            if layer.name.trim().is_empty() {
                return Err(ConfigError::InvalidValue {
                    key: "MEMORY_LAYERS".to_string(),
                    message: "layer name must not be empty".to_string(),
                });
            }
            if layer.name.len() > 64 {
                return Err(ConfigError::InvalidValue {
                    key: "MEMORY_LAYERS".to_string(),
                    message: format!("layer name '{}' exceeds 64 characters", layer.name),
                });
            }
            if !layer
                .name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            {
                return Err(ConfigError::InvalidValue {
                    key: "MEMORY_LAYERS".to_string(),
                    message: format!(
                        "layer name '{}' contains invalid characters (only alphanumeric, _, - allowed)",
                        layer.name
                    ),
                });
            }
            if layer.scope.trim().is_empty() {
                return Err(ConfigError::InvalidValue {
                    key: "MEMORY_LAYERS".to_string(),
                    message: format!("layer '{}' has an empty scope", layer.name),
                });
            }
            if !layer
                .scope
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                return Err(ConfigError::InvalidValue {
                    key: "MEMORY_LAYERS".to_string(),
                    message: format!(
                        "layer '{}' scope '{}' contains invalid characters \
                         (allowed: a-z, A-Z, 0-9, _, -)",
                        layer.name, layer.scope
                    ),
                });
            }
        }

        // Check for duplicate layer names
        {
            let mut seen = std::collections::HashSet::new();
            for layer in &memory_layers {
                if !seen.insert(&layer.name) {
                    return Err(ConfigError::InvalidValue {
                        key: "MEMORY_LAYERS".to_string(),
                        message: format!("duplicate layer name '{}'", layer.name),
                    });
                }
            }
        }

        // --- Read scopes ---
        let read_scopes: Vec<String> = optional_env("WORKSPACE_READ_SCOPES")?
            .map(|s| {
                s.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        for scope in &read_scopes {
            if scope.len() > 128 {
                let prefix: String = scope.chars().take(32).collect();
                return Err(ConfigError::InvalidValue {
                    key: "WORKSPACE_READ_SCOPES".to_string(),
                    message: format!("scope '{prefix}...' exceeds 128 characters"),
                });
            }
            if !scope
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                return Err(ConfigError::InvalidValue {
                    key: "WORKSPACE_READ_SCOPES".to_string(),
                    message: format!(
                        "scope '{}' contains invalid characters \
                         (allowed: a-z, A-Z, 0-9, _, -)",
                        scope
                    ),
                });
            }
        }

        let hdc = HdcConfig::resolve()?;

        Ok(Self {
            memory_layers,
            read_scopes,
            hdc,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::helpers::lock_env;

    fn with_env(key: &str, val: Option<&str>, f: impl FnOnce()) {
        let _guard = lock_env();
        let prev = std::env::var(key).ok();
        match val {
            Some(v) => unsafe { std::env::set_var(key, v) },
            None => unsafe { std::env::remove_var(key) },
        }
        f();
        match prev {
            Some(v) => unsafe { std::env::set_var(key, v) },
            None => unsafe { std::env::remove_var(key) },
        }
    }

    #[test]
    fn valid_json_parses_correctly() {
        let json = r#"[{"name":"private","scope":"alice","writable":true,"sensitivity":"private"},{"name":"shared","scope":"shared","writable":true,"sensitivity":"shared"}]"#;
        with_env("MEMORY_LAYERS", Some(json), || {
            let config = WorkspaceConfig::resolve("alice").expect("should parse");
            assert_eq!(config.memory_layers.len(), 2);
            assert_eq!(config.memory_layers[0].name, "private");
            assert_eq!(config.memory_layers[1].name, "shared");
        });
    }

    #[test]
    fn invalid_json_returns_error() {
        with_env("MEMORY_LAYERS", Some("not json"), || {
            let result = WorkspaceConfig::resolve("alice");
            assert!(result.is_err(), "invalid JSON should fail");
            let err = result.unwrap_err().to_string();
            assert!(
                err.contains("valid JSON"),
                "error should mention JSON: {err}"
            );
        });
    }

    #[test]
    fn empty_layer_name_returns_error() {
        let json = r#"[{"name":"","scope":"alice"}]"#;
        with_env("MEMORY_LAYERS", Some(json), || {
            let result = WorkspaceConfig::resolve("alice");
            assert!(result.is_err(), "empty layer name should fail");
            let err = result.unwrap_err().to_string();
            assert!(err.contains("empty"), "error should mention empty: {err}");
        });
    }

    #[test]
    fn layer_name_exceeding_64_chars_returns_error() {
        let long_name = "a".repeat(65);
        let json = format!(r#"[{{"name":"{long_name}","scope":"alice"}}]"#);
        with_env("MEMORY_LAYERS", Some(&json), || {
            let result = WorkspaceConfig::resolve("alice");
            assert!(result.is_err(), "long layer name should fail");
            let err = result.unwrap_err().to_string();
            assert!(
                err.contains("exceeds 64"),
                "error should mention 64 chars: {err}"
            );
        });
    }

    #[test]
    fn layer_name_with_invalid_chars_returns_error() {
        for bad_name in ["has space", "has@at", "has.dot", "has/slash"] {
            let json = format!(r#"[{{"name":"{bad_name}","scope":"alice"}}]"#);
            with_env("MEMORY_LAYERS", Some(&json), || {
                let result = WorkspaceConfig::resolve("alice");
                assert!(
                    result.is_err(),
                    "layer name '{bad_name}' should fail validation"
                );
                let err = result.unwrap_err().to_string();
                assert!(
                    err.contains("invalid characters"),
                    "error for '{bad_name}' should mention invalid characters: {err}"
                );
            });
        }
    }

    #[test]
    fn empty_scope_returns_error() {
        let json = r#"[{"name":"private","scope":""}]"#;
        with_env("MEMORY_LAYERS", Some(json), || {
            let result = WorkspaceConfig::resolve("alice");
            assert!(result.is_err(), "empty scope should fail");
            let err = result.unwrap_err().to_string();
            assert!(
                err.contains("empty scope"),
                "error should mention empty scope: {err}"
            );
        });
    }

    #[test]
    fn duplicate_layer_names_returns_error() {
        let json = r#"[{"name":"private","scope":"alice"},{"name":"private","scope":"bob"}]"#;
        with_env("MEMORY_LAYERS", Some(json), || {
            let result = WorkspaceConfig::resolve("alice");
            assert!(result.is_err(), "duplicate names should fail");
            let err = result.unwrap_err().to_string();
            assert!(
                err.contains("duplicate"),
                "error should mention duplicate: {err}"
            );
        });
    }

    #[test]
    fn missing_env_defaults_to_single_private_layer() {
        with_env("MEMORY_LAYERS", None, || {
            let config = WorkspaceConfig::resolve("alice").expect("should default");
            assert_eq!(config.memory_layers.len(), 1);
            assert_eq!(config.memory_layers[0].name, "private");
            assert_eq!(config.memory_layers[0].scope, "alice");
            assert!(config.memory_layers[0].writable);
        });
    }
}
