//! Feed discovery helpers for finding GTFS-RT feeds by operator.
//!
//! Provides constants for known European rail operators with GTFS-RT availability.

/// Known European rail operator feeds with GTFS-RT availability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailOperator {
    /// SNCF Transilien/RER (Paris regional rail).
    SncfTransilien,
    /// TER (French regional rail, nationwide).
    TerFrance,
}

impl RailOperator {
    /// Returns the Transitland onestop ID for this operator.
    #[must_use]
    pub const fn onestop_id(self) -> &'static str {
        match self {
            Self::SncfTransilien => "f-u0-sncf~transilien~rer",
            Self::TerFrance => "f-u0-ter",
        }
    }
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
    }
}
