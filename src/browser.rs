use crate::commands::Command;
use headless_chrome::{Browser, LaunchOptionsBuilder, Tab};
use serde_json::Value;
use std::env;
use std::sync::Arc;
use std::time::Duration;

pub trait BrowserExecutor {
    fn execute(&mut self, command: Command) -> Result<String, String>;
}

#[derive(Default)]
pub struct ChromeExecutor {
    browser: Option<Browser>,
    tab: Option<Arc<Tab>>,
}

impl ChromeExecutor {
    fn tab(&self) -> Result<&Tab, String> {
        self.tab
            .as_deref()
            .ok_or_else(|| "no page is open; run `my_browser open <url>` first".to_string())
    }

    fn launch_browser() -> Result<Browser, String> {
        let mut options = LaunchOptionsBuilder::default();
        let headless = !matches!(
            env::var("MY_BROWSER_HEADLESS").as_deref(),
            Ok("0" | "false" | "no")
        );
        options
            .headless(headless)
            .ignore_certificate_errors(false)
            // The library's default disconnects after 30 seconds of inactivity.
            .idle_browser_timeout(Duration::from_secs(365 * 24 * 60 * 60));

        if let Some(path) = env::var_os("MY_BROWSER_CHROME_PATH") {
            options.path(Some(path.into()));
        }

        let options = options
            .build()
            .map_err(|error| format!("invalid Chrome launch options: {error}"))?;
        Browser::new(options).map_err(|error| format!("could not launch Chrome: {error}"))
    }
}

impl BrowserExecutor for ChromeExecutor {
    fn execute(&mut self, command: Command) -> Result<String, String> {
        match command {
            Command::Open { url } => {
                if self.browser.is_none() {
                    self.browser = Some(Self::launch_browser()?);
                }
                if self.tab.is_none() {
                    let tab = self
                        .browser
                        .as_ref()
                        .expect("browser was just launched")
                        .new_tab()
                        .map_err(|error| format!("could not create tab: {error}"))?;
                    self.tab = Some(tab);
                }
                let tab = self.tab()?;
                tab.navigate_to(&url)
                    .and_then(|tab| tab.wait_until_navigated())
                    .map_err(|error| format!("could not open {url}: {error}"))?;
                Ok(tab.get_url())
            }
            Command::Url => Ok(self.tab()?.get_url()),
            Command::Title => self
                .tab()?
                .get_title()
                .map_err(|error| format!("could not read page title: {error}")),
            Command::Snapshot => self
                .tab()?
                .get_content()
                .map_err(|error| format!("could not read page HTML: {error}")),
            Command::Click { target } => {
                self.tab()?
                    .wait_for_element(&target)
                    .and_then(|element| element.click().map(|_| ()))
                    .map_err(|error| format!("could not click {target}: {error}"))?;
                Ok(format!("clicked {target}"))
            }
            Command::Fill { target, text } => {
                let element = self
                    .tab()?
                    .wait_for_element(&target)
                    .map_err(|error| format!("could not find {target}: {error}"))?;
                let result = element
                    .call_js_fn(FILL_SCRIPT, vec![Value::String(text)], false)
                    .map_err(|error| format!("could not fill {target}: {error}"))?;
                if result.value != Some(Value::Bool(true)) {
                    return Err(format!(
                        "cannot fill {target}: target must be an input, textarea, or editable element"
                    ));
                }
                Ok(format!("filled {target}"))
            }
            Command::Close => {
                self.tab = None;
                self.browser = None;
                Ok("browser closed; daemon stopped".to_string())
            }
        }
    }
}

const FILL_SCRIPT: &str = r#"function(value) {
    let setter;
    if (this instanceof HTMLInputElement) {
        setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
    } else if (this instanceof HTMLTextAreaElement) {
        setter = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value').set;
    } else if (this.isContentEditable) {
        this.textContent = value;
    } else {
        return false;
    }
    if (setter) setter.call(this, value);
    this.dispatchEvent(new Event('input', { bubbles: true }));
    this.dispatchEvent(new Event('change', { bubbles: true }));
    return true;
}"#;
