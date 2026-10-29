//! Independently authored Minecraft core. Inputs are official Mojang metadata,
//! not inherited launcher source. Active native bridge uses these modules. Windows/game acceptance remains pending.
pub mod metadata;

pub mod download;

pub mod instances;

pub mod plan;

pub mod install;

pub mod auth;

pub mod launch;

pub mod files;
pub mod java;
pub mod jobs;
pub mod loaders;
pub mod mods;
