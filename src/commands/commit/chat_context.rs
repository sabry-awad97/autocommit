use crate::utils::generate_message;
use crate::{
    commands::config::AutocommitConfig,
    i18n::{self, language::Language},
    utils::{Message, MessageRole},
};
use anyhow::anyhow;
use lazy_static::lazy_static;
use log::{debug, info};
pub struct ChatContext {
    messages: Vec<Message>,
}

impl ChatContext {
    pub fn add_message(&mut self, role: MessageRole, content: String) {
        self.messages.push(Message::new(role, content));
    }

    pub fn get_messages(&self) -> &Vec<Message> {
        &self.messages
    }

    pub fn get_initial_context(config: &AutocommitConfig) -> ChatContext {
        let translation = i18n::get_translation(&Language::English).unwrap();
        let config_data = &config.config_data;
        let emoji_enabled = config_data.emoji_enabled.get_value_ref();
        let description_enabled = config_data.description_enabled.get_value_ref();
        let name = &config_data.name.get_value_ref();
        let email = &config_data.email.get_value_ref();

        let mut system_message = vec![
            "You are to act as the author of a commit message in git.",
            "Your mission is to create clean and comprehensive commit messages in the conventional commit convention and explain why a change was done."
        ];

        if *emoji_enabled {
            system_message.push("Use GitMoji convention to preface the commit.");
        } else {
            system_message.push("Do not preface the commit with anything.");
        }

        if *description_enabled {
            system_message.push("Be specific and concise in the commit message summary, highlighting the most important change(s).");
            system_message.push("Provide more detailed explanation in the commit description, including any relevant context or reasoning behind the change.");
            system_message.push("Don't start it with 'This commit', just describe the changes.");
            system_message.push("Lines must not be longer than 74 characters.");
        } else {
            system_message.push("Don't add any descriptions to the commit, only commit message.")
        }

        system_message.push("Use the present tense.");
        system_message.push("If the change fixes a bug or issue, the type of change is a 'fix'.");
        system_message.push(
            "If the change adds a new feature or enhancement, the type of change is a 'feat'",
        );
        system_message.push("If the change modifies existing functionality, the type of change can be a 'refactor'.");
        system_message.push("If the change modifies existing functionality, the type of change can be a 'refactor'.");
        system_message.push("If the change modifies documentation, updates tests, or makes other minor changes, the type of change is a 'chore'.");
        system_message.push(
            "Use active voice and start with the type of change, such as fix, feat, refactor, etc.",
        );

        let lang = format!("Use {} to answer.", translation.language);
        system_message.push(&lang);
        system_message.push("Be consistent with the formatting and structure of the commit message throughout the commit history.");
        let signed_of_line = format!(
            "Include a 'Signed-off-by: {} <{}>' line indicating the author of the commit.",
            name, email
        );
        system_message.push(&signed_of_line);
        system_message.push("Exclude anything unnecessary such as the original translation—your entire response will be passed directly into git commit.");
        let mut assistant_message = String::new();
        if *emoji_enabled {
            assistant_message.push_str(&format!("🐛 {}\n", translation.commit_fix));
            assistant_message.push_str(&format!("✨ {}\n", translation.commit_feat));
        }
        if *description_enabled {
            assistant_message.push_str(&translation.commit_description);
        }

        let author_line = format!("Signed-off-by: {} <{}>", name, email);
        assistant_message.push_str(&author_line);
        let mut context = ChatContext { messages: vec![] };
        context.add_message(MessageRole::System, system_message.join("\n\n"));
        context.add_message(MessageRole::User, INITIAL_DIFF.to_owned());
        context.add_message(MessageRole::Assistant, assistant_message);

        context
    }

    pub async fn generate_message(&mut self, config: &AutocommitConfig) -> anyhow::Result<String> {
        let open_ai_api_key = config
            .config_data
            .open_ai_api_key
            .get_value_ref()
            .get_inner_value();

        if open_ai_api_key.is_none() {
            return Err(anyhow!("Please set your OpenAI API key in the autocommit config file or as an environment variable"));
        }

        let open_ai_api_key = open_ai_api_key.unwrap();
        let api_host = &config.config_data.api_host.get_value_ref();

        debug!("Generating commit message...");
        let commit_message =
            generate_message(self.get_messages(), &open_ai_api_key, api_host).await?;
        info!("Commit message generated: {}", &commit_message);
        self.add_message(MessageRole::Assistant, commit_message.to_owned());
        Ok(commit_message)
    }
}

lazy_static! {
    pub static ref INITIAL_DIFF: String = String::from(
        r#"
diff --git a/main.rs b/main.rs
index 9a99e25..d6ce76e 100644
--- a/main.rs
+++ b/main.rs
@@ -1,7 +1,6 @@
    use reqwest::Client;
    use serde::{Deserialize, Serialize};
-    use std::{collections::HashMap, env, io::{BufRead, BufReader, stdin, stdout}, process, str::FromStr};
-    use structopt::{clap::arg_enum, StructOpt};
+    use std::{error::Error, io::{self, Write}};
#[derive(Debug, Serialize, Deserialize)]
struct ResponseData {
    joke: String,
}
-    let response = client.get("https://api.icndb.com/jokes/random").send().await?;
+    let response_result = client.get("https://api.icndb.com/jokes/random").send().await;
+
+    let response = match response_result {
+        Ok(resp) => resp,
+        Err(e) => {
+            eprintln!("Error sending request to API: {}", e);
+            std::process::exit(1);
+        }
+    };
+
+    let response_body = response.text().await?;
-    let response_data: ResponseData = serde_json::from_str(&response_body)?;
+    let response_body_trimmed = response_body.trim();
+    let response_data: ResponseData = match serde_json::from_str(response_body_trimmed) {
+        Ok(data) => data,
+        Err(e) => {
+            eprintln!("Error parsing API response: {}", e);
+            std::process::exit(1);
+        }
+    };
+
+    writeln!(io::stdout(), "{}", response_data.joke)?;
+    Ok(())
}    
    "#
    );
}
