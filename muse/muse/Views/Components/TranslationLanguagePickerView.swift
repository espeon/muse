import SwiftUI
import Translation

struct TranslationLanguagePickerView: View {
    @Binding var selectedLanguage: String?
    var onSelect: () -> Void

    @Environment(\.dismiss) private var dismiss
    @State private var supportedLanguages: [Locale.Language] = []

    private let languageAvailability = LanguageAvailability()
    private let englishLocale = Locale(identifier: "en-US")

    private func description(for language: Locale.Language) -> String {
        let code = language.languageCode?.identifier ?? ""
        let name = englishLocale.localizedString(forLanguageCode: code) ?? "Unknown"
        if let region = language.region,
           let regionName = englishLocale.localizedString(forRegionCode: region.identifier) {
            return "\(name), \(regionName)"
        }
        return name
    }

    var body: some View {
        NavigationStack {
            List {
                Section {
                    Button {
                        selectedLanguage = nil
                        onSelect()
                        dismiss()
                    } label: {
                        HStack {
                            Text("Auto (Device Language)")
                            Spacer()
                            if selectedLanguage == nil {
                                Image(systemName: "checkmark")
                                    .foregroundColor(.accentColor)
                            }
                        }
                    }
                }

                Section {
                    ForEach(supportedLanguages, id: \.maximalIdentifier) { language in
                        Button {
                            selectedLanguage = language.minimalIdentifier
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
            .navigationTitle("Translation Language")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Done") { dismiss() }
                }
            }
        }
        .task {
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
