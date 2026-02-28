// TODO this entire parser is in draft state. Missing features are: lossless re-write, all sorts of tests

use std::{
    fmt::Display,
    fs,
    io::{BufRead, BufReader},
    mem,
    path::PathBuf,
};

use anyhow::{Result, bail};
use flexstr::SharedStr;

#[derive(Default)]
struct ConfigFile {
    path: Option<PathBuf>,
    config: Config,
}

impl ConfigFile {
    pub fn load(path: PathBuf) -> Result<Self> {
        let reader = fs::File::open(&path)?;
        let reader = BufReader::new(reader);

        let mut config_builder = ConfigBuilder::default();
        for line in reader.lines() {
            let line = line?;
            config_builder.parse_line(&line)?;
        }
        let config = config_builder.finish()?;

        Ok(Self {
            path: Some(path),
            config,
        })
    }

    // pub fn path(&self) -> Option<&PathBuf> {
    //     self.path.as_ref()
    // }
}

#[derive(Default)]
struct ConfigBuilder {
    config: Config,
    prelude: Vec<String>,
    state: ConfigBuilderState,
    termination_tag: Option<SharedStr>,
}

#[derive(Default)]
enum ConfigBuilderState {
    #[default]
    Default,
    Section {
        key: SharedStr,
        builder: Box<ConfigBuilder>,
    },
    Multiline {
        key: SharedStr,
        multiline: String,
    },
    Complete,
}

impl ConfigBuilder {
    fn new(level: u32) -> Self {
        Self {
            config: Config::new(level),
            prelude: Vec::new(),
            state: ConfigBuilderState::Default,
            termination_tag: None,
        }
    }

    fn parse_line(&mut self, line: &str) -> Result<bool> {
        let trimmed = line.trim();

        self.state = match mem::take(&mut self.state) {
            ConfigBuilderState::Default => {
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    self.prelude.push(line.to_owned());
                    ConfigBuilderState::Default
                } else if self.termination_tag.as_deref() == Some(trimmed) {
                    if !self.prelude.is_empty() {
                        let item = ConfigItem {
                            prelude: mem::take(&mut self.prelude),
                            key_value: None,
                        };
                        self.config.items.push(item);
                    }
                    ConfigBuilderState::Complete
                } else {
                    let Some((key, value)) = trimmed.split_once('=') else {
                        return Err(anyhow::anyhow!("Invalid config line: {line}"));
                    };
                    let key = key.trim();
                    let value = value.trim();

                    if value == "{" {
                        ConfigBuilderState::Section {
                            key: key.to_owned().into(),
                            builder: Box::new(Self::new(self.config.depth + 1)),
                        }
                    } else {
                        let item = ConfigItem {
                            prelude: mem::take(&mut self.prelude),
                            key_value: Some((
                                key.to_owned().into(),
                                ConfigValue::String(value.to_owned().into()),
                            )),
                        };
                        self.config.items.push(item);
                        ConfigBuilderState::Default
                    }
                }
            }
            ConfigBuilderState::Section { key, mut builder } => {
                if builder.parse_line(line)? {
                    let item = ConfigItem {
                        prelude: mem::take(&mut self.prelude),
                        key_value: Some((key, ConfigValue::Group(builder.finish()?))),
                    };
                    self.config.items.push(item);
                    ConfigBuilderState::Default
                } else {
                    ConfigBuilderState::Section { key, builder }
                }
            }
            ConfigBuilderState::Multiline { key, mut multiline } => {
                if line == r#"""""# {
                    let item = ConfigItem {
                        prelude: mem::take(&mut self.prelude),
                        key_value: Some((key.clone(), ConfigValue::String(multiline.into()))),
                    };
                    self.config.items.push(item);
                    ConfigBuilderState::Default
                } else {
                    if !multiline.is_empty() {
                        multiline.push('\n');
                    }
                    multiline.push_str(line);
                    ConfigBuilderState::Multiline { key, multiline }
                }
            }
            ConfigBuilderState::Complete => {
                bail!("Unexpected line after completion");
            }
        };

        Ok(matches!(self.state, ConfigBuilderState::Complete))
    }

    fn finish(mut self) -> Result<Config> {
        match self.state {
            ConfigBuilderState::Default => {
                if !self.prelude.is_empty() {
                    let item = ConfigItem {
                        prelude: mem::take(&mut self.prelude),
                        key_value: None,
                    };
                    self.config.items.push(item);
                }
            }
            ConfigBuilderState::Section { key, builder } => {
                bail!(
                    "missing termination tag for group value of `{key}`: '{}'",
                    builder.termination_tag.unwrap_or_default()
                );
            }
            ConfigBuilderState::Multiline { key, .. } => {
                bail!(r#"missing termination tag for multiline value of `{key}`: '"""'"#);
            }
            ConfigBuilderState::Complete => {}
        }

        Ok(self.config)
    }
}

#[derive(Default)]
struct Config {
    items: Vec<ConfigItem>,
    depth: u32,
}

impl Config {
    fn new(depth: u32) -> Self {
        Self {
            items: Vec::new(),
            depth,
        }
    }
}

impl Display for Config {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in &self.items {
            item.fmt(formatter)?;
        }
        Ok(())
    }
}

struct ConfigItem {
    prelude: Vec<String>,
    key_value: Option<(SharedStr, ConfigValue)>,
    // indent: u32,
}

impl Display for ConfigItem {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in &self.prelude {
            formatter.write_str(line)?;
        }
        if let Some((key, value)) = &self.key_value {
            write!(formatter, "{key} = ")?;
            match value {
                ConfigValue::String(str) => writeln!(formatter, "{str}")?,
                ConfigValue::Group(group) => {
                    writeln!(formatter, "{{")?;
                    for item in &group.items {
                        item.fmt(formatter)?;
                    }
                    writeln!(formatter, "}}")?;
                }
            }
        }
        Ok(())
    }
}

enum ConfigValue {
    String(SharedStr),
    Group(Config),
}
