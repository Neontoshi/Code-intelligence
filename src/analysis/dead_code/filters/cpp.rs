// src/analysis/dead_code/filters/cpp.rs

//! C++-specific dead code filters

use super::common::is_test_file;
use super::{LanguageFilter, ProtectionLevel};
use crate::graph::call_graph::FunctionNode;

pub struct CppFilter;

impl LanguageFilter for CppFilter {
    fn get_protection_level(&self, func: &FunctionNode) -> ProtectionLevel {
        // 1. PROTECTED
        if func.is_test || is_test_file(&func.file) {
            return ProtectionLevel::Protected;
        }

        // Flutter-generated runner boilerplate — GObject vtable callbacks and
        // Win32 WNDPROC entries are wired via macros/function pointers, never
        // called directly, so static reachability can't see their callers.
        // Matched by filename (always auto-generated with these exact names
        // by `flutter create`) rather than directory, since some project
        // layouts nest them under a "runner/" folder and others don't.
        let file_name = func.file.rsplit('/').next().unwrap_or(&func.file);
        if matches!(
            file_name,
            "my_application.cc" | "my_application.h" | "win32_window.cpp" | "win32_window.h"
        ) {
            return ProtectionLevel::Protected;
        }

        // Special member functions and destructors
        if (func.file.ends_with(".cpp")
            || func.file.ends_with(".cc")
            || func.file.ends_with(".hpp")
            || func.file.ends_with(".h"))
            && (func.name.starts_with('~') || func.name == "main" || func.name == "operator=")
        {
            return ProtectionLevel::Protected;
        }

        // 2. LIKELY ALIVE

        // Functions with callers are likely alive
        if func.fan_in > 0 {
            return ProtectionLevel::LikelyAlive;
        }

        // 3. CANDIDATE
        ProtectionLevel::Candidate
    }
}
