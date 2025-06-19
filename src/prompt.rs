// From https://tbaggery.com/2008/04/19/a-note-about-git-commit-messages.html
const GIT_COMMIT_BEST_PRACTICES: &str = r#"
Capitalized, short (50 chars or less) summary

More detailed explanatory text, if necessary.  Wrap it to about 72
characters or so.  In some contexts, the first line is treated as the
subject of an email and the rest of the text as the body.  The blank
line separating the summary from the body is critical (unless you omit
the body entirely); tools like rebase can get confused if you run the
two together.

Write your commit message in the imperative: "Fix bug" and not "Fixed bug"
or "Fixes bug."  This convention matches up with commit messages generated
by commands like git merge and git revert.

Further paragraphs come after blank lines.

- Bullet points are okay, too

- Typically a hyphen or asterisk is used for the bullet, followed by a
  single space, with blank lines in between, but conventions vary here

- Use a hanging indent
"#;

const CONVENTIONAL_COMMIT_ADDITIONAL_RULES: &str = r#"
Follow the Conventional Commits specification: https://www.conventionalcommits.org/en/v1.0.0/

Format: <type>[optional scope]: <description>

[optional body]

[optional footer(s)]

Rules:
- The type and description are mandatory
- The type must be one of: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
- Use 'feat' for new features, 'fix' for bug fixes
- The description must be a short summary in present tense, not capitalized, no period at the end
- Scope is optional and must be a noun in parentheses: fix(parser):
- Body is optional, starts one blank line after description
- Breaking changes: add '!' after type/scope (feat!: or feat(api)!:) OR add 'BREAKING CHANGE:' in the footer
- Footer format: 'token: value' or 'token #value' (e.g., 'Fixes #123' or 'BREAKING CHANGE: description')

Examples:
- feat(auth): add OAuth2 integration
- fix: resolve race condition in data processing
- docs(readme): update installation instructions
- feat(api)!: change response format for /users endpoint
- fix(database): prevent SQL injection in user queries

  BREAKING CHANGE: sanitize() method now returns Promise instead of string

Analyze this git diff and suggest an appropriate conventional commit message:
"#;

pub fn build_prompt(diff: &str, use_conventional: bool) -> String {
    let commit_message_rules = if use_conventional {
        format!(
            "{}\n\n{}",
            GIT_COMMIT_BEST_PRACTICES, CONVENTIONAL_COMMIT_ADDITIONAL_RULES
        )
    } else {
        GIT_COMMIT_BEST_PRACTICES.to_string()
    };

    format!(
        r#"
You are a helpful assistant that generates git commit messages based on code changes.

Please analyze the following git diff and generate a commit message that follows these conventions:

<commit_message_rules>
{}
</commit_message_rules>

Here are the staged changes to analyze:

<git_diff>
{}
</git_diff>

Generate a clear, concise commit message for these changes.
Focus on the "why" and "what" of the changes, not just the "how".
If the changes are simple and self-explanatory, a single line summary is sufficient.

Do not include any other text in your response.
    "#,
        commit_message_rules, diff
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_prompt() -> Result<(), Box<dyn std::error::Error>> {
        let diff = r#"
diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdefg 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -10,6 +10,8 @@ fn main() {
     println!("Hello");
+    let x = 42;
+    println!("The answer is {}", x);
 }
        "#;

        let prompt = build_prompt(diff, false);

        assert!(prompt.contains(
            "You are a helpful assistant that generates git commit messages based on code changes"
        ));
        assert!(prompt.contains(&format!("<git_diff>\n{}\n</git_diff>", diff)));
        assert!(prompt.contains(&format!(
            "<commit_message_rules>\n{}\n</commit_message_rules>",
            GIT_COMMIT_BEST_PRACTICES
        )));
        assert!(prompt.contains("Generate a clear, concise commit message for these changes."));

        Ok(())
    }

    #[test]
    fn test_build_prompt_conventional() -> Result<(), Box<dyn std::error::Error>> {
        let diff = r#"
diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdefg 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -10,6 +10,8 @@ fn main() {
     println!("Hello");
+    let x = 42;
+    println!("The answer is {}", x);
 }
        "#;

        let prompt = build_prompt(diff, true);

        assert!(prompt.contains(
            "You are a helpful assistant that generates git commit messages based on code changes"
        ));
        assert!(prompt.contains(&format!("<git_diff>\n{}\n</git_diff>", diff)));
        assert!(prompt.contains(&format!(
            "<commit_message_rules>\n{}\n\n{}\n</commit_message_rules>",
            GIT_COMMIT_BEST_PRACTICES, CONVENTIONAL_COMMIT_ADDITIONAL_RULES
        )));
        assert!(prompt.contains("Generate a clear, concise commit message for these changes."));

        Ok(())
    }
}
