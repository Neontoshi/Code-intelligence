# 🧹 Dead Code Analysis Report

## 📊 Executive Summary

- **Total Functions**: 789
- **Dead Functions**: 70 (8.9%)
- **Dead Types**: 0
- **Dead Modules**: 0
- **Dead Files**: 0
- **Average Confidence**: 95.7%
- **Estimated LOC Removable**: 207

## 🎯 Priority Removal Order

| # | Function | Confidence | Impact | LOC |
|---|----------|------------|--------|-----|
| 1 | `_triggerBrowserDownload` | 🔴 95-100% | Low impact - 5 LOC, complexity 1.3 | 5 |
| 2 | `_onError` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.3 | 1 |
| 3 | `_startTimer` | 🔴 95-100% | Low impact - 5 LOC, complexity 1.0 | 5 |
| 4 | `_onError` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.3 | 1 |
| 5 | `_onRequest` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.3 | 1 |
| 6 | `_connect` | 🔴 95-100% | Low impact - 4 LOC, complexity 1.0 | 4 |
| 7 | `_onResponse` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.3 | 1 |
| 8 | `_printKV` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.3 | 1 |
| 9 | `_onRequest` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.3 | 1 |
| 10 | `_printAll` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.3 | 1 |
| 11 | `_onResponse` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.3 | 1 |
| 12 | `_selectAdapter` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 13 | `_printRequest` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 14 | `_needRedirect` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 15 | `_isSameOrigin` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 16 | `_init` | 🔴 95-100% | Low impact - 4 LOC, complexity 1.3 | 4 |
| 17 | `_printResponse` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 18 | `_isValidToken` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 19 | `_configHttpClient` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.3 | 1 |
| 20 | `_transformData` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.8 | 1 |
| 21 | `_throwIfCompleted` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 22 | `_dispatchRequest` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 23 | `_fastUtf8JsonDecode` | 🔴 95-100% | Low impact - 4 LOC, complexity 1.3 | 4 |
| 24 | `_fromOptionsAndStream` | 🔴 95-100% | Low impact - 5 LOC, complexity 1.3 | 5 |
| 25 | `_fromOptionsAndStream` | 🔴 95-100% | Low impact - 4 LOC, complexity 1.3 | 4 |
| 26 | `_throwIfH2NotSelected` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 27 | `_contentDispositionKey` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 28 | `_createDownloadAnchor` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 29 | `_fetch` | 🔴 95-100% | Low impact - 5 LOC, complexity 1.6 | 5 |
| 30 | `_fetch` | 🔴 95-100% | Low impact - 6 LOC, complexity 1.6 | 6 |
| 31 | `_defaultValidateStatus` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.3 | 1 |
| 32 | `_handleError` | 🔴 95-100% | Low impact - 4 LOC, complexity 1.3 | 4 |
| 33 | `_handleQueue` | 🔴 95-100% | Low impact - 7 LOC, complexity 1.0 | 7 |
| 34 | `_handleRequest` | 🔴 95-100% | Low impact - 4 LOC, complexity 1.3 | 4 |
| 35 | `_handleResponse` | 🔴 95-100% | Low impact - 4 LOC, complexity 1.3 | 4 |
| 36 | `_createSocket` | 🔴 95-100% | Low impact - 5 LOC, complexity 1.0 | 5 |
| 37 | `_createHttpClient` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 38 | `_getSeparatorChar` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 39 | `_getStreamFromFilepath` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 40 | `_fromSetCookieValue` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.3 | 1 |
| 41 | `_transform` | 🔴 95-100% | Low impact - 5 LOC, complexity 2.3 | 5 |
| 42 | `_transform` | 🔴 95-100% | Low impact - 5 LOC, complexity 2.3 | 5 |
| 43 | `_debugPrint` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.3 | 1 |
| 44 | `_nextRandomId` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 45 | `_revokeObjectUrl` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 46 | `_resolveFilename` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 47 | `_timeoutException` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 48 | `_effectiveU8Encoding` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 49 | `_suggestedFilenameFromPath` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 50 | `_observeInterceptorCallback` | 🔴 95-100% | Low impact - 5 LOC, complexity 1.3 | 5 |
| 51 | `_createObjectUrl` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 52 | `_getCacheKey` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 53 | `_cancelTokenOf` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.6 | 1 |
| 54 | `_browserEncode` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.6 | 1 |
| 55 | `_headerForFile` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.4 | 1 |
| 56 | `_headerForField` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 57 | `_badResponseExceptionMessage` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 58 | `_buildErrorResponse` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 59 | `_buildSuccessResponse` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 60 | `_checkNotNullable` | 🔴 95-100% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 61 | `_invokeCallbackDynamically` | 🔴 95-100% | Low impact - 5 LOC, complexity 1.3 | 5 |
| 62 | `_decodeUtf8ToJson` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.3 | 1 |
| 63 | `_decodeJson` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 64 | `_generateUuid` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 65 | `my_application_dispose` | 🟠 80-95% | Low impact - 5 LOC, complexity 1.2 | 5 |
| 66 | `my_application_activate` | 🟠 80-95% | Medium impact - 46 LOC, complexity 6.4 | 46 |
| 67 | `my_application_init` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 68 | `my_application_class_init` | 🟠 80-95% | Low impact - 5 LOC, complexity 1.0 | 5 |
| 69 | `my_application_local_command_line` | 🟠 80-95% | Low impact - 17 LOC, complexity 1.7 | 17 |
| 70 | `_spawn` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |

## 🔍 Detailed Dead Function Analysis

### 1. `_triggerBrowserDownload`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/plugins/web_adapter/lib/src/download_trigger.dart` (line 47)
- **Confidence**: 99.8%
- **Level**: Guaranteed
- **Complexity**: 1.30
- **Estimated LOC**: 5
- **Dependencies**: 0
- **Impact**: Low impact - 5 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 2. `_onError`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/interceptor.dart` (line 695)
- **Confidence**: 99.7%
- **Level**: Guaranteed
- **Complexity**: 1.30
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 3. `_startTimer`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/plugins/http2_adapter/lib/src/connection_manager_imp.dart` (line 352)
- **Confidence**: 99.7%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 5
- **Dependencies**: 0
- **Impact**: Low impact - 5 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 4. `_onError`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/interceptor.dart` (line 427)
- **Confidence**: 99.7%
- **Level**: Guaranteed
- **Complexity**: 1.30
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 5. `_onRequest`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/interceptor.dart` (line 687)
- **Confidence**: 99.7%
- **Level**: Guaranteed
- **Complexity**: 1.30
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 6. `_connect`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/plugins/http2_adapter/lib/src/connection_manager_imp.dart` (line 107)
- **Confidence**: 99.7%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 4
- **Dependencies**: 0
- **Impact**: Low impact - 4 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 7. `_onResponse`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/interceptor.dart` (line 691)
- **Confidence**: 99.7%
- **Level**: Guaranteed
- **Complexity**: 1.30
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 8. `_printKV`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/interceptors/log.dart` (line 171)
- **Confidence**: 99.7%
- **Level**: Guaranteed
- **Complexity**: 1.30
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 9. `_onRequest`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/interceptor.dart` (line 419)
- **Confidence**: 99.7%
- **Level**: Guaranteed
- **Complexity**: 1.30
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 10. `_printAll`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/interceptors/log.dart` (line 175)
- **Confidence**: 99.7%
- **Level**: Guaranteed
- **Complexity**: 1.30
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 11. `_onResponse`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/interceptor.dart` (line 423)
- **Confidence**: 99.7%
- **Level**: Guaranteed
- **Complexity**: 1.30
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 12. `_selectAdapter`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/plugins/native_dio_adapter/lib/src/cronet_fallback_adapter.dart` (line 136)
- **Confidence**: 99.7%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 13. `_printRequest`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/interceptors/log.dart` (line 101)
- **Confidence**: 99.6%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 14. `_needRedirect`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/plugins/http2_adapter/lib/src/http2_adapter.dart` (line 334)
- **Confidence**: 99.6%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 15. `_isSameOrigin`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/plugins/http2_adapter/lib/src/http2_adapter.dart` (line 352)
- **Confidence**: 99.6%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 16. `_init`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/form_data.dart` (line 47)
- **Confidence**: 99.6%
- **Level**: Guaranteed
- **Complexity**: 1.30
- **Estimated LOC**: 4
- **Dependencies**: 0
- **Impact**: Low impact - 4 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 17. `_printResponse`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/interceptors/log.dart` (line 140)
- **Confidence**: 99.6%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 18. `_isValidToken`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/dio_mixin.dart` (line 668)
- **Confidence**: 99.6%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 19. `_configHttpClient`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/adapters/io_adapter.dart` (line 240)
- **Confidence**: 99.6%
- **Level**: Guaranteed
- **Complexity**: 1.30
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 20. `_transformData`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/dio_mixin.dart` (line 692)
- **Confidence**: 99.6%
- **Level**: Guaranteed
- **Complexity**: 1.80
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.8

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 21. `_throwIfCompleted`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/interceptor.dart` (line 32)
- **Confidence**: 99.6%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 22. `_dispatchRequest`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/dio_mixin.dart` (line 602)
- **Confidence**: 99.6%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 23. `_fastUtf8JsonDecode`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/transformers/fused_transformer.dart` (line 112)
- **Confidence**: 99.6%
- **Level**: Guaranteed
- **Complexity**: 1.30
- **Estimated LOC**: 4
- **Dependencies**: 0
- **Impact**: Low impact - 4 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 24. `_fromOptionsAndStream`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/plugins/native_dio_adapter/lib/src/conversion_layer_adapter.dart` (line 103)
- **Confidence**: 99.6%
- **Level**: Guaranteed
- **Complexity**: 1.30
- **Estimated LOC**: 5
- **Dependencies**: 0
- **Impact**: Low impact - 5 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 25. `_fromOptionsAndStream`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/plugins/compatibility_layer/lib/src/conversion_layer_adapter.dart` (line 46)
- **Confidence**: 99.6%
- **Level**: Guaranteed
- **Complexity**: 1.30
- **Estimated LOC**: 4
- **Dependencies**: 0
- **Impact**: Low impact - 4 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 26. `_throwIfH2NotSelected`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/plugins/http2_adapter/lib/src/connection_manager_imp.dart` (line 319)
- **Confidence**: 99.6%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 27. `_contentDispositionKey`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/form_data.dart` (line 89)
- **Confidence**: 99.6%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 28. `_createDownloadAnchor`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/plugins/web_adapter/lib/src/download_trigger.dart` (line 74)
- **Confidence**: 99.6%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 29. `_fetch`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/adapters/io_adapter.dart` (line 74)
- **Confidence**: 99.6%
- **Level**: Guaranteed
- **Complexity**: 1.60
- **Estimated LOC**: 5
- **Dependencies**: 0
- **Impact**: Low impact - 5 LOC, complexity 1.6

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 30. `_fetch`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/plugins/http2_adapter/lib/src/http2_adapter.dart` (line 63)
- **Confidence**: 99.5%
- **Level**: Guaranteed
- **Complexity**: 1.60
- **Estimated LOC**: 6
- **Dependencies**: 0
- **Impact**: Low impact - 6 LOC, complexity 1.6

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 31. `_defaultValidateStatus`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/options.dart` (line 697)
- **Confidence**: 99.5%
- **Level**: Guaranteed
- **Complexity**: 1.30
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 32. `_handleError`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/interceptor.dart` (line 543)
- **Confidence**: 99.5%
- **Level**: Guaranteed
- **Complexity**: 1.30
- **Estimated LOC**: 4
- **Dependencies**: 0
- **Impact**: Low impact - 4 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 33. `_handleQueue`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/interceptor.dart` (line 565)
- **Confidence**: 99.5%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 7
- **Dependencies**: 0
- **Impact**: Low impact - 7 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 34. `_handleRequest`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/interceptor.dart` (line 503)
- **Confidence**: 99.5%
- **Level**: Guaranteed
- **Complexity**: 1.30
- **Estimated LOC**: 4
- **Dependencies**: 0
- **Impact**: Low impact - 4 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 35. `_handleResponse`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/interceptor.dart` (line 521)
- **Confidence**: 99.4%
- **Level**: Guaranteed
- **Complexity**: 1.30
- **Estimated LOC**: 4
- **Dependencies**: 0
- **Impact**: Low impact - 4 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 36. `_createSocket`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/plugins/http2_adapter/lib/src/connection_manager_imp.dart` (line 177)
- **Confidence**: 99.1%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 5
- **Dependencies**: 0
- **Impact**: Low impact - 5 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 37. `_createHttpClient`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/adapters/io_adapter.dart` (line 257)
- **Confidence**: 99.1%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 38. `_getSeparatorChar`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/utils.dart` (line 130)
- **Confidence**: 99.0%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 39. `_getStreamFromFilepath`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/multipart_file/io_multipart_file.dart` (line 44)
- **Confidence**: 99.0%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 40. `_fromSetCookieValue`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/plugins/cookie_manager/lib/src/cookie_mgr.dart` (line 150)
- **Confidence**: 98.7%
- **Level**: Guaranteed
- **Complexity**: 1.30
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 41. `_transform`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/plugins/web_adapter/lib/src/progress_stream_impl.dart` (line 18)
- **Confidence**: 98.4%
- **Level**: Guaranteed
- **Complexity**: 2.30
- **Estimated LOC**: 5
- **Dependencies**: 0
- **Impact**: Low impact - 5 LOC, complexity 2.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 42. `_transform`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/progress_stream/io_progress_stream.dart` (line 17)
- **Confidence**: 98.4%
- **Level**: Guaranteed
- **Complexity**: 2.30
- **Estimated LOC**: 5
- **Dependencies**: 0
- **Impact**: Low impact - 5 LOC, complexity 2.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 43. `_debugPrint`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/interceptors/log.dart` (line 180)
- **Confidence**: 98.4%
- **Level**: Guaranteed
- **Complexity**: 1.30
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 44. `_nextRandomId`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/form_data.dart` (line 17)
- **Confidence**: 98.3%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 45. `_revokeObjectUrl`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/plugins/web_adapter/lib/src/download_trigger.dart` (line 82)
- **Confidence**: 98.3%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 46. `_resolveFilename`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/plugins/web_adapter/lib/src/dio_impl.dart` (line 92)
- **Confidence**: 98.3%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 47. `_timeoutException`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/compute/compute_web.dart` (line 45)
- **Confidence**: 98.2%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 48. `_effectiveU8Encoding`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/form_data.dart` (line 240)
- **Confidence**: 98.2%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 49. `_suggestedFilenameFromPath`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/plugins/web_adapter/lib/src/dio_impl.dart` (line 102)
- **Confidence**: 98.0%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 50. `_observeInterceptorCallback`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/interceptor.dart` (line 47)
- **Confidence**: 98.0%
- **Level**: Guaranteed
- **Complexity**: 1.30
- **Estimated LOC**: 5
- **Dependencies**: 0
- **Impact**: Low impact - 5 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 51. `_createObjectUrl`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/plugins/web_adapter/lib/src/download_trigger.dart` (line 80)
- **Confidence**: 95.9%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 52. `_getCacheKey`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/plugins/http2_adapter/lib/src/connection_manager_imp.dart` (line 56)
- **Confidence**: 93.8%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - documentation: -0.1
  - ml_prediction: +0.4

### 53. `_cancelTokenOf`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/interceptor.dart` (line 658)
- **Confidence**: 93.6%
- **Level**: Guaranteed
- **Complexity**: 1.60
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.6

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - documentation: -0.1
  - ml_prediction: +0.4

### 54. `_browserEncode`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/form_data.dart` (line 132)
- **Confidence**: 93.6%
- **Level**: Guaranteed
- **Complexity**: 1.60
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.6

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - documentation: -0.1
  - ml_prediction: +0.4

### 55. `_headerForFile`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/form_data.dart` (line 102)
- **Confidence**: 93.6%
- **Level**: Guaranteed
- **Complexity**: 1.40
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.4

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - documentation: -0.1
  - ml_prediction: +0.4

### 56. `_headerForField`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/form_data.dart` (line 94)
- **Confidence**: 93.6%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - documentation: -0.1
  - ml_prediction: +0.4

### 57. `_badResponseExceptionMessage`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/dio_exception.dart` (line 272)
- **Confidence**: 93.5%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - documentation: -0.1
  - ml_prediction: +0.4

### 58. `_buildErrorResponse`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/compute/compute_io.dart` (line 208)
- **Confidence**: 92.8%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - documentation: -0.1
  - ml_prediction: +0.4

### 59. `_buildSuccessResponse`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/compute/compute_io.dart` (line 199)
- **Confidence**: 92.8%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - documentation: -0.1
  - ml_prediction: +0.4

### 60. `_checkNotNullable`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/dio_mixin.dart` (line 867)
- **Confidence**: 92.2%
- **Level**: Guaranteed
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - documentation: -0.1
  - ml_prediction: +0.4

### 61. `_invokeCallbackDynamically`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/interceptor.dart` (line 84)
- **Confidence**: 92.0%
- **Level**: Guaranteed
- **Complexity**: 1.30
- **Estimated LOC**: 5
- **Dependencies**: 0
- **Impact**: Low impact - 5 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - documentation: -0.1
  - ml_prediction: +0.4

### 62. `_decodeUtf8ToJson`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/transformers/fused_transformer.dart` (line 192)
- **Confidence**: 81.2%
- **Level**: VeryLikely
- **Complexity**: 1.30
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - dynamic_reference: -0.4
  - ml_prediction: +0.4

### 63. `_decodeJson`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/transformers/background_transformer.dart` (line 15)
- **Confidence**: 79.9%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - dynamic_reference: -0.4
  - ml_prediction: +0.4

### 64. `_generateUuid`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/example_dart/lib/queued_interceptor_crsftoken.dart` (line 134)
- **Confidence**: 79.6%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 65. `my_application_dispose`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/plugins/native_dio_adapter/example/linux/my_application.cc` (line 85)
- **Confidence**: 78.1%
- **Level**: VeryLikely
- **Complexity**: 1.20
- **Estimated LOC**: 5
- **Dependencies**: 0
- **Impact**: Low impact - 5 LOC, complexity 1.2

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 66. `my_application_activate`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/plugins/native_dio_adapter/example/linux/my_application.cc` (line 18)
- **Confidence**: 78.1%
- **Level**: VeryLikely
- **Complexity**: 6.40
- **Estimated LOC**: 46
- **Dependencies**: 0
- **Impact**: Medium impact - 46 LOC, complexity 6.4

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 67. `my_application_init`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/plugins/native_dio_adapter/example/linux/my_application.cc` (line 97)
- **Confidence**: 78.0%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 68. `my_application_class_init`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/plugins/native_dio_adapter/example/linux/my_application.cc` (line 91)
- **Confidence**: 77.8%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 5
- **Dependencies**: 0
- **Impact**: Low impact - 5 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 69. `my_application_local_command_line`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/plugins/native_dio_adapter/example/linux/my_application.cc` (line 66)
- **Confidence**: 77.8%
- **Level**: VeryLikely
- **Complexity**: 1.70
- **Estimated LOC**: 17
- **Dependencies**: 0
- **Impact**: Low impact - 17 LOC, complexity 1.7

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 70. `_spawn`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/dio/dio/lib/src/compute/compute_io.dart` (line 181)
- **Confidence**: 77.1%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: +0.3
  - is_public: +0.2
  - documentation: -0.1
  - dynamic_reference: -0.4
  - ml_prediction: +0.4

## 💡 Recommendations

1. **Start with high-confidence functions** - Remove functions with Guaranteed/VeryLikely confidence first

---
*Report generated by Code Intelligence Dead Code Analyzer*
