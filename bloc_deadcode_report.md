# 🧹 Dead Code Analysis Report

## 📊 Executive Summary

- **Total Functions**: 919
- **Dead Functions**: 114 (12.4%)
- **Dead Types**: 0
- **Dead Modules**: 0
- **Dead Files**: 0
- **Average Confidence**: 78.2%
- **Estimated LOC Removable**: 365

## 🎯 Priority Removal Order

| # | Function | Confidence | Impact | LOC |
|---|----------|------------|--------|-----|
| 1 | `_buildGenerator` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 2 | `_vars` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 3 | `_subscribe` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 4 | `_close` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 5 | `_report` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 6 | `doOKAction` | 🟠 80-95% | Low impact - 14 LOC, complexity 2.6 | 14 |
| 7 | `_unsubscribe` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 8 | `_dispose` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 9 | `_subscribe` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 10 | `_lineIgnores` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 11 | `_isEndOfLine` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 12 | `_isEndOfLine` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 13 | `_checkForUpdates` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 14 | `_analyzeContent` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 15 | `_ignoresAboveLine` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 16 | `_ignoresAfterLine` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 17 | `_reportDiagnostics` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.3 | 1 |
| 18 | `_analyzeDirectory` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 19 | `hasLatestVersion` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 20 | `_ensureBeforeEndOfLine` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.5 | 1 |
| 21 | `_ensureBeforeEndOfLine` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.5 | 1 |
| 22 | `onGenerateBlocClicked` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 23 | `_maybeStreamIdentical` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 24 | `_updateLatestValue` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.3 | 1 |
| 25 | `_getLineOffsets` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 26 | `_getLineOffsets` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 27 | `getOS` | 🟠 80-95% | Low impact - 12 LOC, complexity 2.2 | 12 |
| 28 | `_getTokens` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 29 | `getArch` | 🟠 80-95% | Low impact - 10 LOC, complexity 2.0 | 10 |
| 30 | `_getFieldName` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 31 | `_getReturnType` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 32 | `createCenterPanel` | 🟠 80-95% | Low impact - 5 LOC, complexity 1.0 | 5 |
| 33 | `targetPath` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 34 | `targetPath` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 35 | `_computeLineOffsets` | 🟠 80-95% | Low impact - 5 LOC, complexity 1.0 | 5 |
| 36 | `_computeLineOffsets` | 🟠 80-95% | Low impact - 5 LOC, complexity 1.0 | 5 |
| 37 | `_cast` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.3 | 1 |
| 38 | `client` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 39 | `blocBuilderSnippet` | 🟠 80-95% | Low impact - 7 LOC, complexity 1.0 | 7 |
| 40 | `_toJson` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.3 | 1 |
| 41 | `options` | 🟠 80-95% | Low impact - 5 LOC, complexity 1.0 | 5 |
| 42 | `options` | 🟠 80-95% | Low impact - 5 LOC, complexity 1.0 | 5 |
| 43 | `options` | 🟠 80-95% | Low impact - 5 LOC, complexity 1.0 | 5 |
| 44 | `filePaths` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 45 | `_observer` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 46 | `_checkCycle` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.3 | 1 |
| 47 | `_removeSeen` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 48 | `_toEncodable` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 49 | `serverOptions` | 🟠 80-95% | Low impact - 9 LOC, complexity 1.0 | 9 |
| 50 | `clientOptions` | 🟠 80-95% | Low impact - 4 LOC, complexity 1.0 | 4 |
| 51 | `_traverseRead` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 52 | `_traverseJson` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 53 | `_traverseWrite` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.3 | 1 |
| 54 | `promptForBlocName` | 🟠 80-95% | Low impact - 7 LOC, complexity 1.0 | 7 |
| 55 | `promptForCubitName` | 🟠 80-95% | Low impact - 7 LOC, complexity 1.0 | 7 |
| 56 | `_traverseAtomicJson` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 57 | `_traverseComplexJson` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 58 | `blocNamePromptOptions` | 🟠 80-95% | Low impact - 4 LOC, complexity 1.0 | 4 |
| 59 | `body` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 60 | `deps` | 🟠 80-95% | Low impact - 10 LOC, complexity 1.0 | 10 |
| 61 | `cubitNamePromptOptions` | 🟠 80-95% | Low impact - 4 LOC, complexity 1.0 | 4 |
| 62 | `action` | 🟠 80-95% | Low impact - 3 LOC, complexity 1.0 | 3 |
| 63 | `hostOS` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 64 | `onError` | 🟠 80-95% | Low impact - 6 LOC, complexity 2.5 | 6 |
| 65 | `devDeps` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 66 | `content` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 67 | `response` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 68 | `hostArch` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 69 | `equatable` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 70 | `DART_FILE` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 71 | `DART_FILE` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 72 | `statusCode` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.6 | 1 |
| 73 | `dependency` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 74 | `executable` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 75 | `childRegExp` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 76 | `openBracket` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 77 | `matchingUris` | 🟠 80-95% | Low impact - 3 LOC, complexity 1.5 | 3 |
| 78 | `closeBracket` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 79 | `blocDirectoryPath` | 🟠 80-95% | Low impact - 3 LOC, complexity 1.3 | 3 |
| 80 | `PUBSPEC_FILE_NAME` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 81 | `snakeCaseBlocName` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 82 | `cubitDirectoryPath` | 🟠 80-95% | Low impact - 3 LOC, complexity 1.3 | 3 |
| 83 | `blocListenerRegExp` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 84 | `blocProviderRegExp` | 🟠 80-95% | Low impact - 4 LOC, complexity 1.0 | 4 |
| 85 | `freezed_annotation` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 86 | `DEFAULT_TIMEOUT_MS` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 87 | `snakeCaseCubitName` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 88 | `DEFAULT_RETRY_COUNT` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 89 | `blocSelectorSnippet` | 🟠 80-95% | Low impact - 10 LOC, complexity 1.0 | 10 |
| 90 | `blocListenerSnippet` | 🟠 80-95% | Low impact - 8 LOC, complexity 1.0 | 8 |
| 91 | `blocProviderSnippet` | 🟠 80-95% | Low impact - 6 LOC, complexity 1.0 | 6 |
| 92 | `blocConsumerSnippet` | 🟠 80-95% | Low impact - 10 LOC, complexity 1.0 | 10 |
| 93 | `installedExecutable` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 94 | `DEFAULT_VERSION_VALUE` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 95 | `interpolatedVarRegExp` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 96 | `ANALYSIS_OPTIONS_FILE` | 🟠 80-95% | Low impact - 4 LOC, complexity 1.0 | 4 |
| 97 | `ANALYSIS_OPTIONS_FILE` | 🟠 80-95% | Low impact - 4 LOC, complexity 1.0 | 4 |
| 98 | `PUBSPEC_LOCK_FILE_NAME` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 99 | `DEFAULT_RETRY_DELAY_MS` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 100 | `openBlocMigrationGuide` | 🟠 80-95% | Low impact - 6 LOC, complexity 1.0 | 6 |
| 101 | `escapedCharacterRegExp` | 🟠 80-95% | Low impact - 1 LOC, complexity 1.0 | 1 |
| 102 | `repositoryProviderRegExp` | 🟠 80-95% | Low impact - 4 LOC, complexity 1.0 | 4 |
| 103 | `currentDependencyVersion` | 🟠 80-95% | Low impact - 4 LOC, complexity 1.0 | 4 |
| 104 | `multiBlocProviderSnippet` | 🟠 80-95% | Low impact - 11 LOC, complexity 1.0 | 11 |
| 105 | `multiBlocListenerSnippet` | 🟠 80-95% | Low impact - 13 LOC, complexity 1.0 | 13 |
| 106 | `repositoryProviderSnippet` | 🟠 80-95% | Low impact - 6 LOC, complexity 1.0 | 6 |
| 107 | `openEquatableMigrationGuide` | 🟠 80-95% | Low impact - 10 LOC, complexity 1.0 | 10 |
| 108 | `multiRepositoryProviderSnippet` | 🟠 80-95% | Low impact - 11 LOC, complexity 1.0 | 11 |
| 109 | `shouldCreateDirectory` | 🟠 80-95% | Low impact - 3 LOC, complexity 1.0 | 3 |
| 110 | `shouldCreateDirectory` | 🟠 80-95% | Low impact - 3 LOC, complexity 1.0 | 3 |
| 111 | `_merge` | 🟠 80-95% | Low impact - 4 LOC, complexity 1.0 | 4 |
| 112 | `promptForTargetDirectory` | 🟠 80-95% | Low impact - 14 LOC, complexity 1.7 | 14 |
| 113 | `promptForTargetDirectory` | 🟠 80-95% | Low impact - 14 LOC, complexity 1.7 | 14 |
| 114 | `futures` | 🟠 80-95% | Low impact - 9 LOC, complexity 1.0 | 9 |

## 🔍 Detailed Dead Function Analysis

### 1. `_buildGenerator`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/bloc_tools/lib/src/commands/new/new_command.dart` (line 95)
- **Confidence**: 79.8%
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

### 2. `_vars`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/bloc_tools/lib/src/commands/new/new_command.dart` (line 77)
- **Confidence**: 79.7%
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

### 3. `_subscribe`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/flutter_bloc/lib/src/bloc_listener.dart` (line 208)
- **Confidence**: 79.7%
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

### 4. `_close`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/bloc/lib/src/emitter.dart` (line 182)
- **Confidence**: 79.7%
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

### 5. `_report`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/bloc_lint/lib/src/rules/avoid_build_context_extensions.dart` (line 81)
- **Confidence**: 79.7%
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

### 6. `doOKAction`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/intellij/intellij_generator_plugin/src/main/java/com/bloc/intellij_generator_plugin/action/GenerateBlocDialog.java` (line 28)
- **Confidence**: 79.7%
- **Level**: VeryLikely
- **Complexity**: 2.60
- **Estimated LOC**: 14
- **Dependencies**: 0
- **Impact**: Low impact - 14 LOC, complexity 2.6

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 7. `_unsubscribe`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/flutter_bloc/lib/src/bloc_listener.dart` (line 218)
- **Confidence**: 79.7%
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

### 8. `_dispose`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/angular_bloc/lib/src/pipes/bloc_pipe.dart` (line 72)
- **Confidence**: 79.7%
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

### 9. `_subscribe`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/angular_bloc/lib/src/pipes/bloc_pipe.dart` (line 58)
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

### 10. `_lineIgnores`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/bloc_lint/lib/src/text_document.dart` (line 73)
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

### 11. `_isEndOfLine`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/bloc_lint/lib/src/text_document.dart` (line 189)
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

### 12. `_isEndOfLine`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/bloc_tools/lib/src/lsp/text_document.dart` (line 191)
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

### 13. `_checkForUpdates`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/bloc_tools/lib/src/command_runner.dart` (line 88)
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

### 14. `_analyzeContent`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/bloc_lint/lib/src/linter.dart` (line 74)
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

### 15. `_ignoresAboveLine`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/bloc_lint/lib/src/text_document.dart` (line 48)
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

### 16. `_ignoresAfterLine`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/bloc_lint/lib/src/text_document.dart` (line 60)
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

### 17. `_reportDiagnostics`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/bloc_tools/lib/src/lsp/language_server.dart` (line 23)
- **Confidence**: 79.6%
- **Level**: VeryLikely
- **Complexity**: 1.30
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 18. `_analyzeDirectory`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/bloc_lint/lib/src/linter.dart` (line 58)
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

### 19. `hasLatestVersion`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/analyze-dependencies.ts` (line 74)
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

### 20. `_ensureBeforeEndOfLine`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/bloc_lint/lib/src/text_document.dart` (line 191)
- **Confidence**: 79.6%
- **Level**: VeryLikely
- **Complexity**: 1.50
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.5

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 21. `_ensureBeforeEndOfLine`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/bloc_tools/lib/src/lsp/text_document.dart` (line 195)
- **Confidence**: 79.6%
- **Level**: VeryLikely
- **Complexity**: 1.50
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.5

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 22. `onGenerateBlocClicked`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/intellij/intellij_generator_plugin/src/main/java/com/bloc/intellij_generator_plugin/action/GenerateBlocDialog.java` (line 50)
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

### 23. `_maybeStreamIdentical`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/angular_bloc/lib/src/pipes/bloc_pipe.dart` (line 82)
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

### 24. `_updateLatestValue`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/angular_bloc/lib/src/pipes/bloc_pipe.dart` (line 65)
- **Confidence**: 79.4%
- **Level**: VeryLikely
- **Complexity**: 1.30
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 25. `_getLineOffsets`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/bloc_lint/lib/src/text_document.dart` (line 160)
- **Confidence**: 79.3%
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

### 26. `_getLineOffsets`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/bloc_tools/lib/src/lsp/text_document.dart` (line 162)
- **Confidence**: 79.3%
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

### 27. `getOS`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/install-bloc-tools.ts` (line 27)
- **Confidence**: 79.1%
- **Level**: VeryLikely
- **Complexity**: 2.20
- **Estimated LOC**: 12
- **Dependencies**: 1
- **Impact**: Low impact - 12 LOC, complexity 2.2

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 28. `_getTokens`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/bloc_lint/lib/src/rules/avoid_public_fields.dart` (line 82)
- **Confidence**: 79.1%
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

### 29. `getArch`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/install-bloc-tools.ts` (line 40)
- **Confidence**: 79.1%
- **Level**: VeryLikely
- **Complexity**: 2.00
- **Estimated LOC**: 10
- **Dependencies**: 1
- **Impact**: Low impact - 10 LOC, complexity 2.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 30. `_getFieldName`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/bloc_lint/lib/src/rules/avoid_public_fields.dart` (line 92)
- **Confidence**: 79.1%
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

### 31. `_getReturnType`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/bloc_lint/lib/src/rules/prefer_void_public_cubit_methods.dart` (line 65)
- **Confidence**: 79.1%
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

### 32. `createCenterPanel`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/intellij/intellij_generator_plugin/src/main/java/com/bloc/intellij_generator_plugin/action/GenerateBlocDialog.java` (line 22)
- **Confidence**: 79.1%
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

### 33. `targetPath`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/new-cubit.command.ts` (line 114)
- **Confidence**: 78.9%
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

### 34. `targetPath`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/new-bloc.command.ts` (line 128)
- **Confidence**: 78.9%
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

### 35. `_computeLineOffsets`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/bloc_lint/lib/src/text_document.dart` (line 165)
- **Confidence**: 78.7%
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

### 36. `_computeLineOffsets`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/bloc_tools/lib/src/lsp/text_document.dart` (line 167)
- **Confidence**: 78.7%
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

### 37. `_cast`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/hydrated_bloc/lib/src/hydrated_bloc.dart` (line 264)
- **Confidence**: 78.5%
- **Level**: VeryLikely
- **Complexity**: 1.30
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 38. `client`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/language-server/language-server.ts` (line 11)
- **Confidence**: 78.5%
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

### 39. `blocBuilderSnippet`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/wrap-with.command.ts` (line 3)
- **Confidence**: 78.5%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 7
- **Dependencies**: 0
- **Impact**: Low impact - 7 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 40. `_toJson`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/hydrated_bloc/lib/src/hydrated_bloc.dart` (line 243)
- **Confidence**: 78.5%
- **Level**: VeryLikely
- **Complexity**: 1.30
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 41. `options`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/downloader.ts` (line 144)
- **Confidence**: 78.5%
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

### 42. `options`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/new-cubit.command.ts` (line 58)
- **Confidence**: 78.5%
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

### 43. `options`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/new-bloc.command.ts` (line 62)
- **Confidence**: 78.5%
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

### 44. `filePaths`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/downloader.ts` (line 93)
- **Confidence**: 78.4%
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

### 45. `_observer`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/replay_bloc/lib/src/replay_bloc.dart` (line 76)
- **Confidence**: 78.4%
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

### 46. `_checkCycle`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/hydrated_bloc/lib/src/hydrated_bloc.dart` (line 348)
- **Confidence**: 78.4%
- **Level**: VeryLikely
- **Complexity**: 1.30
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 47. `_removeSeen`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/hydrated_bloc/lib/src/hydrated_bloc.dart` (line 357)
- **Confidence**: 78.4%
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

### 48. `_toEncodable`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/hydrated_bloc/lib/src/hydrated_bloc.dart` (line 344)
- **Confidence**: 78.4%
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

### 49. `serverOptions`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/language-server/language-server.ts` (line 20)
- **Confidence**: 78.3%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 9
- **Dependencies**: 0
- **Impact**: Low impact - 9 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 50. `clientOptions`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/language-server/language-server.ts` (line 30)
- **Confidence**: 78.3%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 4
- **Dependencies**: 0
- **Impact**: Low impact - 4 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 51. `_traverseRead`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/hydrated_bloc/lib/src/hydrated_bloc.dart` (line 247)
- **Confidence**: 78.3%
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

### 52. `_traverseJson`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/hydrated_bloc/lib/src/hydrated_bloc.dart` (line 336)
- **Confidence**: 78.3%
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

### 53. `_traverseWrite`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/hydrated_bloc/lib/src/hydrated_bloc.dart` (line 266)
- **Confidence**: 78.3%
- **Level**: VeryLikely
- **Complexity**: 1.30
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 54. `promptForBlocName`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/new-bloc.command.ts` (line 53)
- **Confidence**: 78.2%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 7
- **Dependencies**: 0
- **Impact**: Low impact - 7 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 55. `promptForCubitName`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/new-cubit.command.ts` (line 49)
- **Confidence**: 78.2%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 7
- **Dependencies**: 0
- **Impact**: Low impact - 7 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 56. `_traverseAtomicJson`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/hydrated_bloc/lib/src/hydrated_bloc.dart` (line 293)
- **Confidence**: 78.2%
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

### 57. `_traverseComplexJson`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/hydrated_bloc/lib/src/hydrated_bloc.dart` (line 309)
- **Confidence**: 78.2%
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

### 58. `blocNamePromptOptions`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/new-bloc.command.ts` (line 54)
- **Confidence**: 78.1%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 4
- **Dependencies**: 0
- **Impact**: Low impact - 4 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 59. `body`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/downloader.ts` (line 130)
- **Confidence**: 78.1%
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

### 60. `deps`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/analyze-dependencies.ts` (line 37)
- **Confidence**: 78.1%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 10
- **Dependencies**: 0
- **Impact**: Low impact - 10 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 61. `cubitNamePromptOptions`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/new-cubit.command.ts` (line 50)
- **Confidence**: 78.1%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 4
- **Dependencies**: 0
- **Impact**: Low impact - 4 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 62. `action`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/analyze-dependencies.ts` (line 103)
- **Confidence**: 78.1%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 3
- **Dependencies**: 0
- **Impact**: Low impact - 3 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 63. `hostOS`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/install-bloc-tools.ts` (line 28)
- **Confidence**: 78.1%
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

### 64. `onError`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/downloader.ts` (line 131)
- **Confidence**: 78.1%
- **Level**: VeryLikely
- **Complexity**: 2.50
- **Estimated LOC**: 6
- **Dependencies**: 0
- **Impact**: Low impact - 6 LOC, complexity 2.5

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 65. `devDeps`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/analyze-dependencies.ts` (line 48)
- **Confidence**: 78.1%
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

### 66. `content`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/get-pubspec.ts` (line 18)
- **Confidence**: 78.1%
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

### 67. `response`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/downloader.ts` (line 149)
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

### 68. `hostArch`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/install-bloc-tools.ts` (line 41)
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

### 69. `equatable`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/get-bloc-type.ts` (line 8)
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

### 70. `DART_FILE`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/language-server/language-server.ts` (line 13)
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

### 71. `DART_FILE`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/language-server/selectors.ts` (line 5)
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

### 72. `statusCode`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/downloader.ts` (line 132)
- **Confidence**: 78.0%
- **Level**: VeryLikely
- **Complexity**: 1.60
- **Estimated LOC**: 1
- **Dependencies**: 0
- **Impact**: Low impact - 1 LOC, complexity 1.6

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 73. `dependency`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/analyze-dependencies.ts` (line 66)
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

### 74. `executable`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/language-server/language-server.ts` (line 65)
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

### 75. `childRegExp`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/convert-to.ts` (line 4)
- **Confidence**: 77.9%
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

### 76. `openBracket`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/get-selected-text.ts` (line 3)
- **Confidence**: 77.9%
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

### 77. `matchingUris`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/downloader.ts` (line 107)
- **Confidence**: 77.9%
- **Level**: VeryLikely
- **Complexity**: 1.50
- **Estimated LOC**: 3
- **Dependencies**: 0
- **Impact**: Low impact - 3 LOC, complexity 1.5

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 78. `closeBracket`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/get-selected-text.ts` (line 4)
- **Confidence**: 77.9%
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

### 79. `blocDirectoryPath`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/new-bloc.command.ts` (line 84)
- **Confidence**: 77.8%
- **Level**: VeryLikely
- **Complexity**: 1.30
- **Estimated LOC**: 3
- **Dependencies**: 0
- **Impact**: Low impact - 3 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 80. `PUBSPEC_FILE_NAME`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/get-pubspec-path.ts` (line 4)
- **Confidence**: 77.8%
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

### 81. `snakeCaseBlocName`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/new-bloc.command.ts` (line 127)
- **Confidence**: 77.8%
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

### 82. `cubitDirectoryPath`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/new-cubit.command.ts` (line 80)
- **Confidence**: 77.7%
- **Level**: VeryLikely
- **Complexity**: 1.30
- **Estimated LOC**: 3
- **Dependencies**: 0
- **Impact**: Low impact - 3 LOC, complexity 1.3

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 83. `blocListenerRegExp`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/code-actions/bloc-code-action-provider.ts` (line 4)
- **Confidence**: 77.7%
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

### 84. `blocProviderRegExp`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/code-actions/bloc-code-action-provider.ts` (line 5)
- **Confidence**: 77.7%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 4
- **Dependencies**: 0
- **Impact**: Low impact - 4 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 85. `freezed_annotation`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/get-bloc-type.ts` (line 9)
- **Confidence**: 77.7%
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

### 86. `DEFAULT_TIMEOUT_MS`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/downloader.ts` (line 11)
- **Confidence**: 77.7%
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

### 87. `snakeCaseCubitName`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/new-cubit.command.ts` (line 113)
- **Confidence**: 77.7%
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

### 88. `DEFAULT_RETRY_COUNT`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/downloader.ts` (line 12)
- **Confidence**: 77.7%
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

### 89. `blocSelectorSnippet`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/wrap-with.command.ts` (line 11)
- **Confidence**: 77.7%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 10
- **Dependencies**: 0
- **Impact**: Low impact - 10 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 90. `blocListenerSnippet`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/wrap-with.command.ts` (line 22)
- **Confidence**: 77.7%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 8
- **Dependencies**: 0
- **Impact**: Low impact - 8 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 91. `blocProviderSnippet`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/wrap-with.command.ts` (line 31)
- **Confidence**: 77.7%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 6
- **Dependencies**: 0
- **Impact**: Low impact - 6 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 92. `blocConsumerSnippet`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/wrap-with.command.ts` (line 38)
- **Confidence**: 77.7%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 10
- **Dependencies**: 0
- **Impact**: Low impact - 10 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 93. `installedExecutable`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/language-server/language-server.ts` (line 83)
- **Confidence**: 77.7%
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

### 94. `DEFAULT_VERSION_VALUE`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/analyze-dependencies.ts` (line 7)
- **Confidence**: 77.6%
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

### 95. `interpolatedVarRegExp`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/wrap-with.ts` (line 4)
- **Confidence**: 77.6%
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

### 96. `ANALYSIS_OPTIONS_FILE`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/language-server/language-server.ts` (line 14)
- **Confidence**: 77.6%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 4
- **Dependencies**: 0
- **Impact**: Low impact - 4 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 97. `ANALYSIS_OPTIONS_FILE`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/language-server/selectors.ts` (line 1)
- **Confidence**: 77.6%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 4
- **Dependencies**: 0
- **Impact**: Low impact - 4 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 98. `PUBSPEC_LOCK_FILE_NAME`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/get-pubspec-path.ts` (line 5)
- **Confidence**: 77.6%
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

### 99. `DEFAULT_RETRY_DELAY_MS`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/downloader.ts` (line 13)
- **Confidence**: 77.6%
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

### 100. `openBlocMigrationGuide`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/analyze-dependencies.ts` (line 20)
- **Confidence**: 77.6%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 6
- **Dependencies**: 0
- **Impact**: Low impact - 6 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 101. `escapedCharacterRegExp`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/wrap-with.ts` (line 5)
- **Confidence**: 77.6%
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

### 102. `repositoryProviderRegExp`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/code-actions/bloc-code-action-provider.ts` (line 9)
- **Confidence**: 77.5%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 4
- **Dependencies**: 0
- **Impact**: Low impact - 4 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 103. `currentDependencyVersion`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/analyze-dependencies.ts` (line 69)
- **Confidence**: 77.5%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 4
- **Dependencies**: 0
- **Impact**: Low impact - 4 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 104. `multiBlocProviderSnippet`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/convert-to.command.ts` (line 3)
- **Confidence**: 77.5%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 11
- **Dependencies**: 0
- **Impact**: Low impact - 11 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 105. `multiBlocListenerSnippet`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/convert-to.command.ts` (line 15)
- **Confidence**: 77.5%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 13
- **Dependencies**: 0
- **Impact**: Low impact - 13 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 106. `repositoryProviderSnippet`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/wrap-with.command.ts` (line 49)
- **Confidence**: 77.5%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 6
- **Dependencies**: 0
- **Impact**: Low impact - 6 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 107. `openEquatableMigrationGuide`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/analyze-dependencies.ts` (line 26)
- **Confidence**: 77.4%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 10
- **Dependencies**: 0
- **Impact**: Low impact - 10 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 108. `multiRepositoryProviderSnippet`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/convert-to.command.ts` (line 29)
- **Confidence**: 77.3%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 11
- **Dependencies**: 0
- **Impact**: Low impact - 11 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 109. `shouldCreateDirectory`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/new-cubit.command.ts` (line 77)
- **Confidence**: 74.5%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 3
- **Dependencies**: 0
- **Impact**: Low impact - 3 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 110. `shouldCreateDirectory`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/new-bloc.command.ts` (line 81)
- **Confidence**: 74.5%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 3
- **Dependencies**: 0
- **Impact**: Low impact - 3 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 111. `_merge`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/packages/bloc_lint/lib/src/analysis_options.dart` (line 294)
- **Confidence**: 74.5%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 4
- **Dependencies**: 0
- **Impact**: Low impact - 4 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - documentation: -0.1
  - ml_prediction: +0.4

### 112. `promptForTargetDirectory`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/new-cubit.command.ts` (line 57)
- **Confidence**: 72.6%
- **Level**: VeryLikely
- **Complexity**: 1.70
- **Estimated LOC**: 14
- **Dependencies**: 0
- **Impact**: Low impact - 14 LOC, complexity 1.7

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 113. `promptForTargetDirectory`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/commands/new-bloc.command.ts` (line 61)
- **Confidence**: 72.6%
- **Level**: VeryLikely
- **Complexity**: 1.70
- **Estimated LOC**: 14
- **Dependencies**: 0
- **Impact**: Low impact - 14 LOC, complexity 1.7

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

### 114. `futures`

- **File**: `/home/dicey/Documents/Core/training_repos/dart/bloc/extensions/vscode/src/utils/analyze-dependencies.ts` (line 116)
- **Confidence**: 70.0%
- **Level**: VeryLikely
- **Complexity**: 1.00
- **Estimated LOC**: 9
- **Dependencies**: 0
- **Impact**: Low impact - 9 LOC, complexity 1.0

**Factors:**
  - fan_in: +0.4
  - reachability: -0.3
  - is_public: +0.2
  - ml_prediction: +0.4

## 💡 Recommendations

1. **Start with high-confidence functions** - Remove functions with Guaranteed/VeryLikely confidence first

---
*Report generated by Code Intelligence Dead Code Analyzer*
