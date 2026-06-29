import Combine
import Foundation
import Translation

@MainActor
class TranslationService: ObservableObject {
    @Published var isTranslating = false

    /// Translate multiple lyric lines using the provided session
    /// Batches all lines together with blank line separators for better language detection
    func translateBatch(_ texts: [String], session: TranslationSession) async throws -> [String] {
        // Filter out empty lines and track their indices
        var nonEmptyIndices: [Int] = []
        var nonEmptyTexts: [String] = []

        for (index, text) in texts.enumerated() {
            if !text.isEmpty {
                nonEmptyIndices.append(index)
                nonEmptyTexts.append(text)
            }
        }

        guard !nonEmptyTexts.isEmpty else {
            return texts
        }

        // Batch all non-empty lines together with blank line separators
        let batchedText = nonEmptyTexts.joined(separator: "\n\n")

        do {
            let response = try await session.translate(batchedText)

            // Check if source and target are the same language
            if response.sourceLanguage == response.targetLanguage {
                // No translation needed
                return texts
            }

            // Split the result back up
            let translatedLines = response.targetText.components(separatedBy: "\n\n")

            // Build final results array
            var results: [String] = Array(repeating: "", count: texts.count)
            for (arrayIndex, originalIndex) in nonEmptyIndices.enumerated() {
                if arrayIndex < translatedLines.count {
                    results[originalIndex] = translatedLines[arrayIndex]
                } else {
                    // Fallback if split didn't work as expected
                    results[originalIndex] = texts[originalIndex]
                }
            }

            return results
        } catch {
            // If translation fails, keep original text
            return texts
        }
    }
}
