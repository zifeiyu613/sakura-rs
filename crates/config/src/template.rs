use std::collections::HashMap;
use crate::ConfigError;

pub struct TemplateEngine {
    variables: HashMap<String, String>,
    start_delim: String,
    end_delim: String,
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            start_delim: "${".to_string(),
            end_delim: "}".to_string(),
        }
    }

    pub fn with_delimiters<S: Into<String>>(mut self, start: S, end: S) -> Self {
        self.start_delim = start.into();
        self.end_delim = end.into();
        self
    }

    pub fn set_variable<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) -> &mut Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    pub fn set_variables<K: Into<String>, V: Into<String>>(
        &mut self,
        vars: impl IntoIterator<Item = (K, V)>
    ) -> &mut Self {
        for (key, value) in vars {
            self.variables.insert(key.into(), value.into());
        }
        self
    }

    pub fn process(&self, template: &str) -> Result<String, ConfigError> {
        let mut result = template.to_string();
        let mut processed = false;

        // 循环直到所有变量都被替换
        while !processed {
            processed = true;

            for (key, value) in &self.variables {
                let placeholder = format!("{}{}{}", self.start_delim, key, self.end_delim);

                if result.contains(&placeholder) {
                    result = result.replace(&placeholder, value);
                    processed = false; // 仍然有变量被替换，可能需要再次处理（嵌套变量的情况）
                }
            }
        }

        // 检查是否有未替换的变量
        let start_delim_esc = regex::escape(&self.start_delim);
        let end_delim_esc = regex::escape(&self.end_delim);
        let pattern = format!(r"{}\w+{}", start_delim_esc, end_delim_esc);

        if let Ok(re) = regex::Regex::new(&pattern) {
            if re.is_match(&result) {
                // 找出第一个未替换的变量
                if let Some(capture) = re.find(&result) {
                    return Err(ConfigError::Other(format!(
                        "Unresolved template variable: {}",
                        capture.as_str()
                    )));
                }
            }
        }

        Ok(result)
    }

    // 将模板引擎应用于配置文件内容
    pub fn process_config_file(&self, content: &str) -> Result<String, ConfigError> {
        self.process(content)
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

// 扩展FileLoader以支持模板处理
impl crate::loader::FileLoader {
    pub fn with_template_engine(mut self, engine: &TemplateEngine) -> Self {
        // 在FileLoader中保存模板引擎的引用
        // 注意：这需要FileLoader结构体增加一个字段
        self
    }
}
