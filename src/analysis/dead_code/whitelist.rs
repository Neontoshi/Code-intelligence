// src/analysis/dead_code/whitelist.rs

use std::collections::HashSet;
use std::sync::LazyLock;

/// Global whitelist of functions that should never be considered dead
/// even if they have no callers (entry points, FFI exports, etc.)
pub static WHITELIST: LazyLock<Whitelist> = LazyLock::new(Whitelist::new);

pub struct Whitelist {
    functions: HashSet<String>,
    patterns: Vec<String>,
}

impl Whitelist {
    pub fn new() -> Self {
        let mut functions = HashSet::new();
        let mut patterns = Vec::new();

        // ================================================================
        // ENTRY POINTS
        // ================================================================
        functions.insert("main".to_string());
        functions.insert("async_main".to_string());
        functions.insert("run".to_string());
        functions.insert("start".to_string());
        functions.insert("init".to_string());

        // ================================================================
        // FFI / EXPORTED SYMBOLS
        // ================================================================
        functions.insert("java_".to_string()); // JNI
        functions.insert("JNI_".to_string());
        functions.insert("Python_".to_string()); // Python C API
        functions.insert("node_".to_string()); // Node.js N-API
        functions.insert("napi_".to_string());

        // ================================================================
        // LIBRARY ENTRY POINTS
        // ================================================================
        functions.insert("new".to_string());
        functions.insert("create".to_string());
        functions.insert("build".to_string());

        // ================================================================
        // COMMON TRAIT/INTERFACE METHODS
        // ================================================================
        functions.insert("generate".to_string());
        functions.insert("generate_stream".to_string());
        functions.insert("model_name".to_string());
        functions.insert("max_context_length".to_string());
        functions.insert("is_available".to_string());
        functions.insert("default".to_string());
        functions.insert("clone".to_string());
        functions.insert("drop".to_string());

        // ================================================================
        // GO INTERFACE METHODS (jsoniter)
        // ================================================================
        functions.insert("ValueType".to_string());
        functions.insert("MustBeValid".to_string());
        functions.insert("LastError".to_string());
        functions.insert("ToBool".to_string());
        functions.insert("ToInt".to_string());
        functions.insert("ToInt32".to_string());
        functions.insert("ToInt64".to_string());
        functions.insert("ToUint".to_string());
        functions.insert("ToUint32".to_string());
        functions.insert("ToUint64".to_string());
        functions.insert("ToFloat32".to_string());
        functions.insert("ToFloat64".to_string());
        functions.insert("ToString".to_string());
        functions.insert("WriteTo".to_string());
        functions.insert("GetInterface".to_string());
        functions.insert("Parse".to_string());
        functions.insert("Get".to_string());
        functions.insert("Size".to_string());
        functions.insert("Keys".to_string());
        functions.insert("ToVal".to_string());
        functions.insert("ReadArray".to_string());
        functions.insert("ReadArrayCB".to_string());
        functions.insert("ReadObject".to_string());
        functions.insert("ReadObjectCB".to_string());
        functions.insert("ReadMapCB".to_string());

        // ================================================================
        // KNOWN INTERNAL FUNCTIONS (Used via reflection/function pointers)
        // ================================================================

        // Stream internal helpers
        functions.insert("writeByte".to_string());
        functions.insert("writeTwoBytes".to_string());
        functions.insert("writeThreeBytes".to_string());
        functions.insert("writeFourBytes".to_string());
        functions.insert("writeFiveBytes".to_string());
        functions.insert("writeIndention".to_string());

        // Float parsing internal helpers
        functions.insert("readPositiveFloat32".to_string());
        functions.insert("readNumberAsString".to_string());
        functions.insert("readFloat32SlowPath".to_string());
        functions.insert("readPositiveFloat64".to_string());
        functions.insert("readFloat64SlowPath".to_string());

        // Object parsing internal helpers
        functions.insert("readFieldHash".to_string());
        functions.insert("readObjectStart".to_string());
        functions.insert("readObjectFieldAsBytes".to_string());

        // Decoding internal helpers
        functions.insert("doDecode".to_string());
        functions.insert("decodeOneField".to_string());

        // Any type helpers
        functions.insert("readAny".to_string());
        functions.insert("readNumberAny".to_string());
        functions.insert("readObjectAny".to_string());
        functions.insert("readArrayAny".to_string());

        // Reflection helpers
        functions.insert("caseSensitive".to_string());

        // Iterator internal helpers
        functions.insert("skipWhitespacesWithoutLoadMore".to_string());
        functions.insert("isObjectEnd".to_string());
        functions.insert("nextToken".to_string());
        functions.insert("readByte".to_string());
        functions.insert("loadMore".to_string());
        functions.insert("unreadByte".to_string());
        functions.insert("incrementDepth".to_string());
        functions.insert("decrementDepth".to_string());

        // Skip helpers (sloppy/strict)
        functions.insert("skipNumber".to_string());
        functions.insert("skipArray".to_string());
        functions.insert("skipObject".to_string());
        functions.insert("skipString".to_string());
        functions.insert("findStringEnd".to_string());
        functions.insert("trySkipNumber".to_string());
        functions.insert("trySkipString".to_string());

        // Capture helpers
        functions.insert("startCaptureTo".to_string());
        functions.insert("startCapture".to_string());
        functions.insert("stopCapture".to_string());
        functions.insert("skipFourBytes".to_string());
        functions.insert("skipThreeBytes".to_string());

        // String parsing helpers
        functions.insert("readStringSlowPath".to_string());
        functions.insert("readEscapedChar".to_string());
        functions.insert("readU4".to_string());

        // Integer parsing helpers
        functions.insert("readUint32".to_string());
        functions.insert("readUint64".to_string());
        functions.insert("assertInteger".to_string());

        // Cache management helpers
        functions.insert("initCache".to_string());
        functions.insert("addDecoderToCache".to_string());
        functions.insert("addEncoderToCache".to_string());
        functions.insert("getDecoderFromCache".to_string());
        functions.insert("getEncoderFromCache".to_string());
        functions.insert("frozeWithCacheReuse".to_string());
        functions.insert("validateJsonRawMessage".to_string());
        functions.insert("getTagKey".to_string());
        functions.insert("marshalFloatWith6Digits".to_string());
        functions.insert("escapeHTML".to_string());
        functions.insert("cleanDecoders".to_string());
        functions.insert("cleanEncoders".to_string());

        // ================================================================
        // KNOWN INTERNAL FUNCTIONS (from other languages/projects)
        // ================================================================
        functions.insert("compress_text".to_string());
        functions.insert("encode_hex".to_string());
        functions.insert("get_languages".to_string());
        functions.insert("get_key_files".to_string());

        // ================================================================
        // TEST HELPERS
        // ================================================================
        functions.insert("create_test_function".to_string());

        // ================================================================
        // ⭐ NEW: Chart.js internal functions (used by the library)
        // ================================================================
        functions.insert("_computeGridLineItems".to_string());
        functions.insert("_computeLabelItems".to_string());
        functions.insert("calculateLabelRotation".to_string());
        functions.insert("_computeLabelSizes".to_string());
        functions.insert("_getLabelSizes".to_string());
        functions.insert("getLabelForValue".to_string());
        functions.insert("getPixelForValue".to_string());
        functions.insert("getValueForPixel".to_string());
        functions.insert("getPixelForTick".to_string());
        functions.insert("getPixelForDecimal".to_string());
        functions.insert("getDecimalForPixel".to_string());
        functions.insert("getBasePixel".to_string());
        functions.insert("getBaseValue".to_string());
        functions.insert("_tickSize".to_string());
        functions.insert("_isVisible".to_string());
        functions.insert("drawBackground".to_string());
        functions.insert("drawGrid".to_string());
        functions.insert("drawBorder".to_string());
        functions.insert("drawLabels".to_string());
        functions.insert("drawTitle".to_string());
        functions.insert("_layers".to_string());
        functions.insert("getMatchingVisibleMetas".to_string());
        functions.insert("_resolveTickFontOptions".to_string());
        functions.insert("_maxDigits".to_string());
        functions.insert("register".to_string());
        functions.insert("unregister".to_string());
        functions.insert("notify".to_string());
        functions.insert("invalidate".to_string());
        functions.insert("_descriptors".to_string());
        functions.insert("_createDescriptors".to_string());
        functions.insert("platform".to_string());
        functions.insert("registry".to_string());
        functions.insert("_initialize".to_string());
        functions.insert("clear".to_string());
        functions.insert("resize".to_string());
        functions.insert("_resize".to_string());
        functions.insert("ensureScalesHaveIDs".to_string());
        functions.insert("buildOrUpdateScales".to_string());
        functions.insert("_updateMetasets".to_string());
        functions.insert("buildOrUpdateControllers".to_string());
        functions.insert("_resetElements".to_string());
        functions.insert("_updateScales".to_string());
        functions.insert("_checkEventBindings".to_string());
        functions.insert("_updateHiddenIndices".to_string());
        functions.insert("_getUniformDataChanges".to_string());
        functions.insert("_updateLayout".to_string());
        functions.insert("_updateDatasets".to_string());
        functions.insert("_updateDataset".to_string());
        functions.insert("render".to_string());
        functions.insert("draw".to_string());
        functions.insert("_getSortedDatasetMetas".to_string());
        functions.insert("getSortedVisibleDatasetMetas".to_string());
        functions.insert("_drawDatasets".to_string());
        functions.insert("_drawDataset".to_string());
        functions.insert("isPointInArea".to_string());
        functions.insert("getElementsAtEventForMode".to_string());
        functions.insert("getDatasetMeta".to_string());
        functions.insert("getContext".to_string());
        functions.insert("getVisibleDatasetCount".to_string());
        functions.insert("isDatasetVisible".to_string());
        functions.insert("toggleDataVisibility".to_string());
        functions.insert("getDataVisibility".to_string());
        functions.insert("_updateVisibility".to_string());
        functions.insert("hide".to_string());
        functions.insert("_destroyDatasetMeta".to_string());
        functions.insert("_stop".to_string());
        functions.insert("destroy".to_string());
        functions.insert("toBase64Image".to_string());
        functions.insert("bindEvents".to_string());
        functions.insert("bindUserEvents".to_string());
        functions.insert("bindResponsiveEvents".to_string());
        functions.insert("unbindEvents".to_string());
        functions.insert("updateHoverStyle".to_string());
        functions.insert("getActiveElements".to_string());
        functions.insert("notifyPlugins".to_string());
        functions.insert("isPluginEnabled".to_string());
        functions.insert("_updateHoverStyles".to_string());
        functions.insert("_eventHandler".to_string());
        functions.insert("_handleEvent".to_string());
        functions.insert("_getActiveElements".to_string());
        functions.insert("updateControlPoints".to_string());
        functions.insert("interpolate".to_string());
        functions.insert("pathSegment".to_string());
        functions.insert("inRange".to_string());
        functions.insert("inXRange".to_string());
        functions.insert("inYRange".to_string());
        functions.insert("getCenterPoint".to_string());
        functions.insert("size".to_string());
        functions.insert("getRange".to_string());
        functions.insert("afterDatasetsUpdate".to_string());
        functions.insert("beforeDraw".to_string());
        functions.insert("beforeDatasetsDraw".to_string());
        functions.insert("beforeDatasetDraw".to_string());
        functions.insert("average".to_string());
        functions.insert("nearest".to_string());

        // ================================================================
        // PATTERNS
        // ================================================================
        patterns.push("^test_".to_string());
        patterns.push("^bench_".to_string());
        patterns.push("^mock_".to_string());
        patterns.push("_test$".to_string());
        patterns.push("_bench$".to_string());

        Self {
            functions,
            patterns,
        }
    }

    /// Check if a function is whitelisted by exact name
    pub fn is_whitelisted(&self, name: &str) -> bool {
        // Check exact matches
        if self.functions.contains(name) {
            return true;
        }

        // Check pattern matches
        for pattern in &self.patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(name) {
                    return true;
                }
            }
        }

        false
    }

    /// Check if a function is whitelisted by full path
    pub fn is_whitelisted_path(&self, full_path: &str) -> bool {
        // ⭐ NEW: Skip bundled/minified JavaScript files
        if full_path.ends_with(".js") && full_path.contains("/dist/") {
            return true;
        }
        if full_path.ends_with(".js") && full_path.contains("/build/") {
            return true;
        }
        if full_path.ends_with(".js") && full_path.contains("/assets/") {
            return true;
        }
        if full_path.ends_with(".min.js") {
            return true;
        }
        if full_path.contains(".bundle.js") {
            return true;
        }

        // ⭐ NEW: Skip node_modules
        if full_path.contains("node_modules/") {
            return true;
        }

        // ⭐ NEW: Skip compiled/generated JS files
        if full_path.ends_with(".js")
            && (full_path.contains("browser-")
                || full_path.contains("main-")
                || full_path.contains("index-")
                || full_path.contains("app-")
                || full_path.contains("client-")
                || full_path.contains("remote-")
                || full_path.contains("chunk-")
                || full_path.contains("butterchunk-"))
        {
            return true;
        }

        // Skip example solutions in .meta directories
        if full_path.contains("/.meta/") {
            return true;
        }

        // Skip generated files
        if full_path.contains(".gen.go") || full_path.contains("_gen.go") {
            return true;
        }

        // Skip generated test files
        if full_path.contains("cases_test.go") {
            return true;
        }

        // Check if it's in the benchmark directory
        if full_path.contains("/benches/") {
            return true;
        }

        // Check if it's in the tests directory
        if full_path.contains("/tests/") || full_path.ends_with("_test.rs") {
            return true;
        }

        // Check if it's a build script
        if full_path.contains("build.rs") {
            return true;
        }

        false
    }

    /// Add a function to the whitelist dynamically
    pub fn add_function(&mut self, name: &str) {
        self.functions.insert(name.to_string());
    }

    /// Remove a function from the whitelist
    pub fn remove_function(&mut self, name: &str) {
        self.functions.remove(name);
    }
}

impl Default for Whitelist {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whitelist_exact_match() {
        let whitelist = Whitelist::new();
        assert!(whitelist.is_whitelisted("main"));
        assert!(whitelist.is_whitelisted("generate"));
        assert!(whitelist.is_whitelisted("compress_text"));
        assert!(whitelist.is_whitelisted("writeByte"));
        assert!(whitelist.is_whitelisted("readEscapedChar"));
        assert!(whitelist.is_whitelisted("skipNumber"));
        assert!(!whitelist.is_whitelisted("random_function"));
    }

    #[test]
    fn test_whitelist_pattern_match() {
        let whitelist = Whitelist::new();
        assert!(whitelist.is_whitelisted("test_my_function"));
        assert!(whitelist.is_whitelisted("bench_my_function"));
        assert!(whitelist.is_whitelisted("my_function_test"));
        assert!(!whitelist.is_whitelisted("regular_function"));
    }

    #[test]
    fn test_whitelist_path_match() {
        let whitelist = Whitelist::new();
        assert!(whitelist.is_whitelisted_path("src/benches/compression.rs"));
        assert!(whitelist.is_whitelisted_path("src/tests/test_self_analysis.rs"));
        assert!(whitelist.is_whitelisted_path("src/build.rs"));
        assert!(whitelist.is_whitelisted_path("exercises/practice/ledger/.meta/example.go"));
        assert!(!whitelist.is_whitelisted_path("src/engine/pipeline.rs"));
    }

    #[test]
    fn test_bundled_js_filter() {
        let whitelist = Whitelist::new();

        // Bundled JS files should be filtered out
        assert!(whitelist.is_whitelisted_path("/dist/assets/browser-BXdiCFWD.js"));
        assert!(whitelist.is_whitelisted_path("/dist/assets/main-06ciBZDq.js"));
        assert!(whitelist.is_whitelisted_path("/build/assets/app-ByPOcLMs.js"));
        assert!(whitelist.is_whitelisted_path("/assets/index-0pYbquBB.js"));
        assert!(whitelist.is_whitelisted_path("/assets/butterchunk-CMvS5UXf.js"));

        // Minified JS files should be filtered out
        assert!(whitelist.is_whitelisted_path("/jquery.min.js"));
        assert!(whitelist.is_whitelisted_path("/lodash.min.js"));

        // Node modules should be filtered out
        assert!(whitelist.is_whitelisted_path("/node_modules/react/index.js"));
        assert!(whitelist.is_whitelisted_path("/node_modules/chart.js/dist/chart.min.js"));

        // Regular JS files should NOT be filtered out
        assert!(!whitelist.is_whitelisted_path("/src/components/Button.tsx"));
        assert!(!whitelist.is_whitelisted_path("/src/services/logger.ts"));
        assert!(!whitelist.is_whitelisted_path("/main.js"));
    }

    #[test]
    fn test_go_internal_helpers_whitelisted() {
        let whitelist = Whitelist::new();

        // Stream helpers
        assert!(whitelist.is_whitelisted("writeByte"));
        assert!(whitelist.is_whitelisted("writeTwoBytes"));
        assert!(whitelist.is_whitelisted("writeThreeBytes"));
        assert!(whitelist.is_whitelisted("writeFourBytes"));
        assert!(whitelist.is_whitelisted("writeFiveBytes"));

        // Skip helpers
        assert!(whitelist.is_whitelisted("skipNumber"));
        assert!(whitelist.is_whitelisted("skipArray"));
        assert!(whitelist.is_whitelisted("skipObject"));
        assert!(whitelist.is_whitelisted("skipString"));

        // String parsing helpers
        assert!(whitelist.is_whitelisted("readEscapedChar"));
        assert!(whitelist.is_whitelisted("readStringSlowPath"));

        // Cache helpers
        assert!(whitelist.is_whitelisted("initCache"));
        assert!(whitelist.is_whitelisted("addDecoderToCache"));
        assert!(whitelist.is_whitelisted("cleanDecoders"));
    }

    #[test]
    fn test_chartjs_functions_whitelisted() {
        let whitelist = Whitelist::new();

        // Chart.js internal functions should be whitelisted
        assert!(whitelist.is_whitelisted("_computeGridLineItems"));
        assert!(whitelist.is_whitelisted("_computeLabelItems"));
        assert!(whitelist.is_whitelisted("calculateLabelRotation"));
        assert!(whitelist.is_whitelisted("getPixelForValue"));
        assert!(whitelist.is_whitelisted("draw"));
        assert!(whitelist.is_whitelisted("register"));
        assert!(whitelist.is_whitelisted("unregister"));
        assert!(whitelist.is_whitelisted("destroy"));
    }

    #[test]
    fn test_meta_directory_filter() {
        let whitelist = Whitelist::new();

        // These should be filtered out (example solutions)
        assert!(whitelist.is_whitelisted_path("/exercises/practice/ledger/.meta/example.go"));
        assert!(whitelist.is_whitelisted_path("/exercises/practice/forth/.meta/example.go"));
        assert!(whitelist.is_whitelisted_path("/exercises/practice/bowling/.meta/example.go"));

        // These should NOT be filtered out (actual code)
        assert!(!whitelist.is_whitelisted_path("/exercises/practice/ledger/ledger.go"));
        assert!(!whitelist.is_whitelisted_path("/exercises/practice/forth/forth.go"));
        assert!(!whitelist.is_whitelisted_path("/exercises/practice/bowling/bowling.go"));
    }
}
