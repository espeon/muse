import Foundation

enum LLMTranslationError: LocalizedError {
    case notConfigured
    case noAPIKey
    case requestFailed(Int, String)
    case noContent
    case invalidJSON(String)
    case lineCountMismatch(expected: Int, got: Int)

    var errorDescription: String? {
        switch self {
        case .notConfigured:
            "LLM translation is not configured. Add an API key in Settings."
        case .noAPIKey:
            "No API key set."
        case .requestFailed(let code, let body):
            "API request failed (\(code)): \(body)"
        case .noContent:
            "The API returned an empty response."
        case .invalidJSON(let detail):
            "The API returned invalid JSON: \(detail)"
        case .lineCountMismatch(let expected, let got):
            "Translation line count mismatch: expected \(expected), got \(got)."
        }
    }
}

struct LLMTranslationConfig {
    let apiKey: String
    let baseURL: String
    let model: String

    var isConfigured: Bool { !apiKey.isEmpty }

    static var current: LLMTranslationConfig {
        LLMTranslationConfig(
            apiKey: UserDefaults.standard.string(forKey: "muse.llm.apiKey") ?? "",
            baseURL: UserDefaults.standard.string(forKey: "muse.llm.baseURL")
                ?? "https://openrouter.ai/api/v1",
            model: UserDefaults.standard.string(forKey: "muse.llm.model") ?? "openai/gpt-4o-mini"
        )
    }
}

struct LLMTranslationService {

    func translate(texts: [String], targetLanguage: String, config: LLMTranslationConfig)
        async throws -> [String]
    {
        guard config.isConfigured else { throw LLMTranslationError.notConfigured }

        var nonEmptyIndices: [Int] = []
        var nonEmptyTexts: [String] = []
        for (index, text) in texts.enumerated() {
            if !text.isEmpty {
                nonEmptyIndices.append(index)
                nonEmptyTexts.append(text)
            }
        }
        guard !nonEmptyTexts.isEmpty else { return texts }

        let translatedLines = try await callAPI(
            texts: nonEmptyTexts,
            targetLanguage: targetLanguage,
            config: config
        )

        guard translatedLines.count == nonEmptyTexts.count else {
            throw LLMTranslationError.lineCountMismatch(
                expected: nonEmptyTexts.count,
                got: translatedLines.count
            )
        }

        var results = Array(repeating: "", count: texts.count)
        for (arrayIndex, originalIndex) in nonEmptyIndices.enumerated() {
            results[originalIndex] = translatedLines[arrayIndex]
        }
        return results
    }

    private func callAPI(texts: [String], targetLanguage: String, config: LLMTranslationConfig)
        async throws -> [String]
    {
        let systemPrompt = """
            You are a professional lyric translator. You will receive a JSON array of lyric lines, \
            in their original sequential order. Translate each line into \(targetLanguage), \
            taking into account surrounding lines for context. Try to make it flow well in the target \
            language, even if that means deviating from a literal translation. Preserve any rhymes or \
            poetic devices where possible, but prioritize flow, naturalness and emotional impact in the target \
            language.

            Your response must be ONLY a JSON array of strings, with exactly \(texts.count) elements, \
            in the same order as the input. No markdown, no explanation, just the array.

            These are song lyrics presented in the order they are sung. Preserve that order exactly; \
            do not rearrange, merge, or skip lines.

            Example input: ["hello world", "goodbye"]
            Example output: ["hola mundo", "adiós"]
            """

        let jsonData = try JSONEncoder().encode(texts)
        let userContent = String(data: jsonData, encoding: .utf8) ?? "[]"

        struct Message: Encodable {
            let role: String
            let content: String
        }
        struct RequestBody: Encodable {
            let model: String
            let messages: [Message]
            let temperature: Double
        }
        struct ResponseBody: Decodable {
            struct Choice: Decodable {
                struct Message: Decodable { let content: String? }
                let message: Message
            }
            let choices: [Choice]?
        }

        let body = RequestBody(
            model: config.model,
            messages: [
                Message(role: "system", content: systemPrompt),
                Message(role: "user", content: userContent),
            ],
            temperature: 0.3
        )

        let bodyData = try JSONEncoder().encode(body)
        let urlString =
            config.baseURL.trimmingCharacters(in: CharacterSet(charactersIn: "/"))
            + "/chat/completions"
        guard let url = URL(string: urlString) else {
            throw LLMTranslationError.requestFailed(0, "Invalid URL: \(urlString)")
        }

        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("Bearer \(config.apiKey)", forHTTPHeaderField: "Authorization")
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = bodyData
        request.timeoutInterval = 60

        let (data, response): (Data, URLResponse)
        do {
            (data, response) = try await URLSession.shared.data(for: request)
        } catch {
            throw LLMTranslationError.requestFailed(0, error.localizedDescription)
        }

        guard let httpResponse = response as? HTTPURLResponse else {
            throw LLMTranslationError.requestFailed(0, "Invalid response")
        }

        guard (200...299).contains(httpResponse.statusCode) else {
            let body = String(data: data, encoding: .utf8) ?? ""
            throw LLMTranslationError.requestFailed(httpResponse.statusCode, body)
        }

        let decoded = try JSONDecoder().decode(ResponseBody.self, from: data)
        guard let content = decoded.choices?.first?.message.content, !content.isEmpty else {
            throw LLMTranslationError.noContent
        }

        return try parseResponse(content)
    }

    private func parseResponse(_ content: String) throws -> [String] {
        var cleaned = content.trimmingCharacters(in: .whitespacesAndNewlines)

        if cleaned.hasPrefix("```") {
            if let firstNewline = cleaned.firstIndex(of: "\n") {
                cleaned = String(cleaned[cleaned.index(after: firstNewline)...])
            }
            if let closingRange = cleaned.range(of: "\n```", options: .backwards) {
                cleaned = String(cleaned[..<closingRange.lowerBound])
            } else if cleaned.hasSuffix("```") {
                cleaned = String(cleaned.dropLast(3))
            }
            cleaned = cleaned.trimmingCharacters(in: .whitespacesAndNewlines)
        }

        if let data = cleaned.data(using: .utf8),
            let lines = try? JSONDecoder().decode([String].self, from: data)
        {
            return lines
        }

        if let firstBracket = cleaned.firstIndex(of: "["),
            let lastBracket = cleaned.lastIndex(of: "]")
        {
            let jsonString = String(cleaned[firstBracket...lastBracket])
            if let data = jsonString.data(using: .utf8),
                let lines = try? JSONDecoder().decode([String].self, from: data)
            {
                return lines
            }
        }

        throw LLMTranslationError.invalidJSON(String(content.prefix(200)))
    }
}
