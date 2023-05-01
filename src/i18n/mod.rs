use std::collections::HashMap;

use self::language::Language;

pub mod language;

#[derive(Debug, Clone)]
pub struct I18n {
    translations: HashMap<Language, I18nTranslation>,
}

#[derive(Debug, Clone)]
pub struct I18nTranslation {
    pub commit_fix: String,
    pub commit_feat: String,
    pub commit_description: String,
    pub language: String,
}

impl I18n {
    pub fn new(translations: HashMap<Language, I18nTranslation>) -> Self {
        Self { translations }
    }

    pub fn get(&self, local: &Language) -> Option<&I18nTranslation> {
        self.translations.get(local)
    }
}

pub fn load_i18n() -> I18n {
    let mut translations = HashMap::new();
    translations.insert(
        Language::English,
        I18nTranslation {
            commit_fix: "fix(main.rs): Correct JSON parsing issue for joke response ".to_string(),
            commit_feat: "feat(main.rs): Add error handling for API request ".to_string(),
            commit_description: String::from(format!("After further testing, it was determined that JSON response data for the joke endpoint contained leading/trailing white space. To fix the issue, string trimming was added to the JSON parsing step.
            To improve the error handling logic of the API request, a `match` expression was added to handle the case when the API request fails.
            Updates:
            - The `serde_json::from_str` function now uses `trim()` function to remove leading/trailing spaces before data parsing. 
            - A `match` expression now handles the `Err` case when making the API request.")),
            language: "English".to_string(),
        },
    );

    I18n::new(translations)
}

pub fn get_translation(local: &Language) -> Option<I18nTranslation> {
    let i18n = load_i18n();
    let translation = i18n.get(local).cloned();
    translation
}
