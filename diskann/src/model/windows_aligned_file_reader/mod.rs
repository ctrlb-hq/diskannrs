/*
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT license.
 */
#[cfg(target_os = "windows")]
#[allow(clippy::module_inception)]
mod windows_aligned_file_reader;
#[cfg(target_os = "windows")]
pub use windows_aligned_file_reader::*;
