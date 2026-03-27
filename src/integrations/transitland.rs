//! Transitland GTFS-RT API client for rail journey status enrichment.
//!
//! Provides access to real-time transit data via Transitland's REST API v2,
//! which aggregates GTFS-RT feeds from operators worldwide. This module focuses
//! on rail operators with available real-time feeds.
//!
//! # Architecture
//!
//! - `client`: HTTP client with Transitland API key authentication
//! - `feed_discovery`: Query `/feeds` endpoint to find GTFS-RT feeds by operator
//! - `gtfs_rt`: Protobuf parsing for GTFS-RT TripUpdate messages
//! - `matcher`: Logic to match journey records to trip_id and extract delays
//! - `cache`: Static GTFS feed caching and trip_id indexing
//!
//! # Supported European Rail Operators
//!
//! - **France**: SNCF Transilien/RER (`f-u0-sncf~transilien~rer`), TER (`f-u0-ter`)
//! - **Netherlands**: NS via OVapi direct (not Transitland)
//! - **Italy**: Trenitalia (static only, no RT)
//! - **Spain**: Renfe (static only, no RT)
//! - **Switzerland**: SBB (static only, no RT)

/// Static GTFS feed caching and indexing.
pub mod cache;
/// HTTP client for Transitland REST API v2.
pub mod client;
/// Feed discovery endpoint wrapper.
pub mod feed_discovery;
/// GTFS-RT protobuf parsing.
pub mod gtfs_rt;
/// Journey → trip_id → delay matching logic.
pub mod matcher;
/// [`RailStatusApi`] trait implementation for Transitland GTFS-RT.
pub mod rail_status_impl;
