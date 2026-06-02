//! Scripting bridges: native Rust, Python (PyO3), and Java (JNI/GraalVM).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptLang {
    Rust,
    Python,
    Java,
}

pub struct ScriptHost {
    pub loaded: Vec<(ScriptLang, String)>,
}

impl ScriptHost {
    pub fn new() -> Self {
        Self { loaded: Vec::new() }
    }

    pub fn load(&mut self, lang: ScriptLang, path: impl Into<String>) {
        self.loaded.push((lang, path.into()));
    }

    pub fn update(&mut self, _dt: f32) {
        // TODO: dispatch update() to each script via its bridge
    }
}

impl Default for ScriptHost {
    fn default() -> Self {
        Self::new()
    }
}
