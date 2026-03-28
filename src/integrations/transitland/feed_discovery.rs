//! Feed discovery helpers for finding GTFS-RT feeds by operator.
//!
//! Provides constants for known European rail operators with GTFS-RT availability.

use super::client::{FeedSearchParams, TransitlandClient};

/// Known European rail operator feeds with GTFS-RT availability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailOperator {
    /// SNCF Transilien/RER (Paris regional rail).
    SncfTransilien,
    /// TER (French regional rail, nationwide).
    TerFrance,
    /// Trenitalia France (Thello) — Paris-Milan.
    TrenitaliaFrance,
}

impl RailOperator {
    /// Returns the Transitland onestop ID for this operator.
    #[must_use]
    pub const fn onestop_id(self) -> &'static str {
        match self {
            Self::SncfTransilien => "f-u0-sncf~transilien~rer",
            Self::TerFrance => "f-u0-ter",
            Self::TrenitaliaFrance => "f-thello~rt",
        }
    }

    /// Returns a human-readable display name.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::SncfTransilien => "SNCF Transilien/RER",
            Self::TerFrance => "TER France",
            Self::TrenitaliaFrance => "Trenitalia France (Thello)",
        }
    }

    /// Returns the country code for this operator.
    #[must_use]
    pub const fn country_code(self) -> &'static str {
        match self {
            Self::SncfTransilien | Self::TerFrance | Self::TrenitaliaFrance => "FR",
        }
    }
}

/// Discovers GTFS-RT feeds for a specific operator.
///
/// # Errors
///
/// Returns an error if the API request fails.
pub async fn discover_feeds_for_operator(
    client: &TransitlandClient,
    operator: RailOperator,
) -> Result<super::client::FeedSearchResponse, reqwest::Error> {
    let params = FeedSearchParams {
        spec: Some("gtfs-rt".to_string()),
        onestop_id: Some(operator.onestop_id().to_string()),
        search: None,
    };

    client.search_feeds(&params).await
}

/// Returns all supported rail operators with GTFS-RT availability.
#[must_use]
pub fn supported_operators() -> Vec<RailOperator> {
    vec![
        RailOperator::SncfTransilien,
        RailOperator::TerFrance,
        RailOperator::TrenitaliaFrance,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operator_onestop_ids_are_correct() {
        assert_eq!(
            RailOperator::SncfTransilien.onestop_id(),
            "f-u0-sncf~transilien~rer"
        );
        assert_eq!(RailOperator::TerFrance.onestop_id(), "f-u0-ter");
        assert_eq!(RailOperator::TrenitaliaFrance.onestop_id(), "f-thello~rt");
    }

    #[test]
    fn all_operators_have_country_code() {
        for operator in supported_operators() {
            assert!(!operator.country_code().is_empty());
        }
    }
}
