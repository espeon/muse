import SwiftUI
import Translation

struct LLMLanguage: Identifiable, Hashable {
    let code: String
    let name: String
    var id: String { code }
}

private let llmLanguages: [LLMLanguage] = [
    .init(code: "af", name: "Afrikaans"),
    .init(code: "ar", name: "Arabic"),
    .init(code: "be", name: "Belarusian"),
    .init(code: "bg", name: "Bulgarian"),
    .init(code: "bn", name: "Bengali"),
    .init(code: "ca", name: "Catalan"),
    .init(code: "cs", name: "Czech"),
    .init(code: "cy", name: "Welsh"),
    .init(code: "da", name: "Danish"),
    .init(code: "de", name: "German"),
    .init(code: "el", name: "Greek"),
    .init(code: "en", name: "English"),
    .init(code: "eo", name: "Esperanto"),
    .init(code: "es", name: "Spanish"),
    .init(code: "et", name: "Estonian"),
    .init(code: "fa", name: "Persian"),
    .init(code: "fi", name: "Finnish"),
    .init(code: "fr", name: "French"),
    .init(code: "ga", name: "Irish"),
    .init(code: "gl", name: "Galician"),
    .init(code: "gu", name: "Gujarati"),
    .init(code: "he", name: "Hebrew"),
    .init(code: "hi", name: "Hindi"),
    .init(code: "hr", name: "Croatian"),
    .init(code: "ht", name: "Haitian"),
    .init(code: "hu", name: "Hungarian"),
    .init(code: "id", name: "Indonesian"),
    .init(code: "is", name: "Icelandic"),
    .init(code: "it", name: "Italian"),
    .init(code: "ja", name: "Japanese"),
    .init(code: "ka", name: "Georgian"),
    .init(code: "kn", name: "Kannada"),
    .init(code: "ko", name: "Korean"),
    .init(code: "lt", name: "Lithuanian"),
    .init(code: "lv", name: "Latvian"),
    .init(code: "mk", name: "Macedonian"),
    .init(code: "mr", name: "Marathi"),
    .init(code: "ms", name: "Malay"),
    .init(code: "mt", name: "Maltese"),
    .init(code: "nl", name: "Dutch"),
    .init(code: "no", name: "Norwegian"),
    .init(code: "pl", name: "Polish"),
    .init(code: "pt", name: "Portuguese"),
    .init(code: "ro", name: "Romanian"),
    .init(code: "ru", name: "Russian"),
    .init(code: "sk", name: "Slovak"),
    .init(code: "sl", name: "Slovenian"),
    .init(code: "sq", name: "Albanian"),
    .init(code: "sv", name: "Swedish"),
    .init(code: "sw", name: "Swahili"),
    .init(code: "ta", name: "Tamil"),
    .init(code: "te", name: "Telugu"),
    .init(code: "th", name: "Thai"),
    .init(code: "tl", name: "Tagalog"),
    .init(code: "tr", name: "Turkish"),
    .init(code: "uk", name: "Ukrainian"),
    .init(code: "ur", name: "Urdu"),
    .init(code: "vi", name: "Vietnamese"),
    .init(code: "zh-Hans", name: "Chinese (Simplified)"),
    .init(code: "zh-Hant", name: "Chinese (Traditional)"),
]

struct TranslationLanguagePickerView: View {
    @Binding var selectedLanguage: String?
    var useLLM: Binding<Bool>?
    var llmLanguage: Binding<String>?
    var llmConfigured: Bool
    var onSelect: () -> Void

    @Environment(\.dismiss) private var dismiss
    @State private var supportedLanguages: [Locale.Language] = []
    @State private var mode: Int = 0
    @State private var searchQuery = ""
    @State private var recentLanguages: [String] = []

    private let languageAvailability = LanguageAvailability()
    private let englishLocale = Locale(identifier: "en-US")

    private static let recentKey = "muse.llm.recentLanguages"

    private func description(for language: Locale.Language) -> String {
        let code = language.languageCode?.identifier ?? ""
        let name = englishLocale.localizedString(forLanguageCode: code) ?? "Unknown"
        if let region = language.region,
           let regionName = englishLocale.localizedString(forRegionCode: region.identifier) {
            return "\(name), \(regionName)"
        }
        return name
    }

    private var filteredLLMLanguages: [LLMLanguage] {
        guard !searchQuery.isEmpty else { return llmLanguages }
        return llmLanguages.filter { $0.name.localizedCaseInsensitiveContains(searchQuery) }
    }

    private var recentLLMLanguages: [LLMLanguage] {
        recentLanguages.compactMap { name in
            llmLanguages.first { $0.name == name }
        }
    }

    private var filteredDeviceLanguages: [Locale.Language] {
        guard !searchQuery.isEmpty else { return supportedLanguages }
        return supportedLanguages.filter {
            description(for: $0).localizedCaseInsensitiveContains(searchQuery)
        }
    }

    private func selectLLMLanguage(_ name: String) {
        useLLM?.wrappedValue = true
        llmLanguage?.wrappedValue = name

        var recent = recentLanguages
        recent.removeAll { $0 == name }
        recent.insert(name, at: 0)
        if recent.count > 6 { recent = Array(recent.prefix(6)) }
        UserDefaults.standard.set(recent, forKey: Self.recentKey)
        recentLanguages = recent

        onSelect()
        dismiss()
    }

    var body: some View {
        NavigationStack {
            List {
                if llmConfigured {
                    Picker("Method", selection: $mode) {
                        Text("Device").tag(0)
                        Text("LLM").tag(1)
                    }
                    .pickerStyle(.segmented)
                    .listRowBackground(Color.clear)
                    .listRowInsets(EdgeInsets(top: 4, leading: 16, bottom: 4, trailing: 16))
                }

                if llmConfigured && mode == 1 {
                    if !recentLLMLanguages.isEmpty && searchQuery.isEmpty {
                        Section("Recent") {
                            ForEach(recentLLMLanguages) { language in
                                Button {
                                    selectLLMLanguage(language.name)
                                } label: {
                                    HStack {
                                        Text(language.name)
                                        Spacer()
                                        if llmLanguage?.wrappedValue == language.name {
                                            Image(systemName: "checkmark")
                                                .foregroundColor(.accentColor)
                                        }
                                    }
                                }
                                .foregroundColor(.primary)
                            }
                        }
                    }

                    Section {
                        ForEach(filteredLLMLanguages) { language in
                            Button {
                                selectLLMLanguage(language.name)
                            } label: {
                                HStack {
                                    Text(language.name)
                                    Spacer()
                                    if llmLanguage?.wrappedValue == language.name {
                                        Image(systemName: "checkmark")
                                            .foregroundColor(.accentColor)
                                    }
                                }
                            }
                            .foregroundColor(.primary)
                        }
                    } header: {
                        Text("All Languages")
                    } footer: {
                        Text("LLM-powered translation. Works with any language.")
                    }
                } else {
                    Section {
                        Button {
                            selectedLanguage = nil
                            useLLM?.wrappedValue = false
                            onSelect()
                            dismiss()
                        } label: {
                            HStack {
                                Text("Auto (Device Language)")
                                Spacer()
                                if selectedLanguage == nil && useLLM?.wrappedValue != true {
                                    Image(systemName: "checkmark")
                                        .foregroundColor(.accentColor)
                                }
                            }
                        }

                        ForEach(filteredDeviceLanguages, id: \.maximalIdentifier) { language in
                            Button {
                                selectedLanguage = language.minimalIdentifier
                                useLLM?.wrappedValue = false
                                onSelect()
                                dismiss()
                            } label: {
                                HStack {
                                    Text(description(for: language))
                                    Spacer()
                                    if selectedLanguage == language.minimalIdentifier {
                                        Image(systemName: "checkmark")
                                            .foregroundColor(.accentColor)
                                    }
                                }
                            }
                            .foregroundColor(.primary)
                        }
                    } header: {
                        Text("Available Languages")
                    } footer: {
                        Text(
                            "Only languages downloaded on your device are shown. Download more in Settings > General > Language & Region > Translation Languages."
                        )
                    }
                }
            }
            .searchable(text: $searchQuery, prompt: "Search languages")
            .navigationTitle("Translation Language")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Done") { dismiss() }
                }
            }
        }
        .task {
            if useLLM?.wrappedValue == true { mode = 1 }
            recentLanguages = UserDefaults.standard.stringArray(forKey: Self.recentKey) ?? []
            let supported = await languageAvailability.supportedLanguages
            for language in supported {
                let status = await languageAvailability.status(from: language, to: englishLocale.language)
                if status == .installed {
                    supportedLanguages.append(language)
                }
            }
        }
    }
}
