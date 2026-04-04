#![allow(clippy::must_use_candidate, clippy::needless_pass_by_value)]

//! Leptos SSR components — reusable UI building blocks for rendered HTML pages.

pub(crate) mod attachment_gallery;
pub(crate) mod auth_page;
pub(crate) mod carrier_icon;
pub(crate) mod error_page;
pub(crate) mod format_utils;
pub(crate) mod map_controls;
pub(crate) mod navbar;
pub(crate) mod overview_cards;
pub(crate) mod shell;
pub(crate) mod stats_filters;
pub(crate) mod top_list;

pub(crate) use auth_page::AuthFormPage;
pub(crate) use carrier_icon::CarrierIcon;
pub(crate) use error_page::ErrorPage;
pub(crate) use navbar::NavBar;
pub(crate) use shell::Shell;
