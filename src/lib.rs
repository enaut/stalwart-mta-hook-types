/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 * SPDX-FileCopyrightText: 2025 Franz Dietrich <dietrich@teilgedanken.de>
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

pub mod modifications;
pub mod request;
pub mod response;

pub use modifications::*;
pub use request::*;
pub use response::*;

// Type aliases for backward compatibility
pub type MtaHookResponse = Response;
