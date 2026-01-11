use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

pub type XoxResult<T> = Result<T, XoxError>;

#[derive(Debug, Clone)]
pub enum XoxError {
    ParseError(String),
    WriteError(String),
    LinkError(String),
    WorkspaceError(String),
    GitError(String),
    DependencyError(String),
    LockError(String),
    ConfigNotFoundError(String),
    MultipleConfigFilesError(String),
}

impl std::fmt::Display for XoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XoxError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            XoxError::WriteError(msg) => write!(f, "Write error: {}", msg),
            XoxError::LinkError(msg) => write!(f, "Link error: {}", msg),
            XoxError::WorkspaceError(msg) => write!(f, "Workspace error: {}", msg),
            XoxError::GitError(msg) => write!(f, "Git error: {}", msg),
            XoxError::DependencyError(msg) => write!(f, "Dependency error: {}", msg),
            XoxError::LockError(msg) => write!(f, "Lock error: {}", msg),
            XoxError::ConfigNotFoundError(msg) => write!(f, "Config file not found: {}", msg),
            XoxError::MultipleConfigFilesError(msg) => write!(f, "Multiple config files found: {}", msg),
        }
    }
}

impl std::error::Error for XoxError {}

#[derive(Debug, Clone, PartialEq)]
pub enum XoxValue {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Binary(Vec<u8>),
    List(Vec<XoxValue>),
    Map(HashMap<String, XoxValue>),
}

impl XoxValue {
    pub fn null() -> Self {
        XoxValue::Null
    }

    pub fn bool(b: bool) -> Self {
        XoxValue::Bool(b)
    }

    pub fn int(n: i64) -> Self {
        XoxValue::Integer(n)
    }

    pub fn float(f: f64) -> Self {
        XoxValue::Float(f)
    }

    pub fn string(s: String) -> Self {
        XoxValue::String(s)
    }

    pub fn binary(data: Vec<u8>) -> Self {
        XoxValue::Binary(data)
    }

    pub fn list(list: Vec<XoxValue>) -> Self {
        XoxValue::List(list)
    }

    pub fn map(map: HashMap<String, XoxValue>) -> Self {
        XoxValue::Map(map)
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            XoxValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            XoxValue::Integer(i) => Some(*i),
            XoxValue::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            XoxValue::Float(f) => Some(*f),
            XoxValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        match self {
            XoxValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&Vec<XoxValue>> {
        match self {
            XoxValue::List(list) => Some(list),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&HashMap<String, XoxValue>> {
        match self {
            XoxValue::Map(map) => Some(map),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&XoxValue> {
        match self {
            XoxValue::Map(map) => map.get(key),
            _ => None,
        }
    }

    pub fn index(&self, idx: usize) -> Option<&XoxValue> {
        match self {
            XoxValue::List(list) => list.get(idx),
            _ => None,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            XoxValue::List(list) => list.len(),
            XoxValue::Map(map) => map.len(),
            XoxValue::String(s) => s.len(),
            XoxValue::Binary(data) => data.len(),
            _ => 0,
        }
    }
}

impl From<String> for XoxValue {
    fn from(s: String) -> Self {
        XoxValue::String(s)
    }
}

impl From<&str> for XoxValue {
    fn from(s: &str) -> Self {
        XoxValue::String(s.to_string())
    }
}

impl From<i64> for XoxValue {
    fn from(i: i64) -> Self {
        XoxValue::Integer(i)
    }
}

impl From<f64> for XoxValue {
    fn from(f: f64) -> Self {
        XoxValue::Float(f)
    }
}

impl From<bool> for XoxValue {
    fn from(b: bool) -> Self {
        XoxValue::Bool(b)
    }
}

impl From<Vec<XoxValue>> for XoxValue {
    fn from(list: Vec<XoxValue>) -> Self {
        XoxValue::List(list)
    }
}

impl From<HashMap<String, XoxValue>> for XoxValue {
    fn from(map: HashMap<String, XoxValue>) -> Self {
        XoxValue::Map(map)
    }
}

#[derive(Debug, Clone)]
pub struct XoxDocument {
    pub root: XoxValue,
    pub metadata: HashMap<String, String>,
}

impl XoxDocument {
    pub fn new() -> Self {
        XoxDocument {
            root: XoxValue::Null,
            metadata: HashMap::new(),
        }
    }

    pub fn set_root(&mut self, root: XoxValue) {
        self.root = root;
    }

    pub fn get_root(&self) -> &XoxValue {
        &self.root
    }

    pub fn get_root_mut(&mut self) -> &mut XoxValue {
        &mut self.root
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    pub fn to_string(&self) -> XoxResult<String> {
        let mut output = String::new();
        self.write(&mut output)?;
        Ok(output)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> XoxResult<()> {
        writeln!(writer, "# Xox Package Configuration v1.0")?;

        for (key, value) in &self.metadata {
            writeln!(writer, "# {}: {}", key, value)?;
        }

        self.write_value(writer, &self.root, 0)?;
        Ok(())
    }

    fn write_value<W: Write>(
        &self,
        writer: &mut W,
        value: &XoxValue,
        depth: usize,
    ) -> XoxResult<()> {
        let indent = " ".repeat(depth * 2);

        match value {
            XoxValue::Null => writeln!(writer, "{}null", indent)?,
            XoxValue::Bool(b) => writeln!(writer, "{}{}", indent, if *b { "true" } else { "false" })?,
            XoxValue::Integer(i) => writeln!(writer, "{}i:{}", indent, i)?,
            XoxValue::Float(f) => writeln!(writer, "{}f:{}", indent, f)?,
            XoxValue::String(s) => {
                if s.contains('\n') || s.contains('"') {
                    writeln!(writer, "{}s:|", indent)?;
                    for line in s.lines() {
                        writeln!(writer, "{}  {}", indent, line)?;
                    }
                } else {
                    writeln!(writer, '{}s:"{}"', indent, s)?;
                }
            }
            XoxValue::Binary(data) => {
                writeln!(writer, "{}b:{}", indent, base64::encode(data))?;
            }
            XoxValue::List(list) => {
                writeln!(writer, "{}[", indent)?;
                for v in list {
                    self.write_value(writer, v, depth + 1)?;
                }
                writeln!(writer, "{}]", indent)?;
            }
            XoxValue::Map(map) => {
                writeln!(writer, "{{", indent)?;
                for (k, v) in map {
                    write!(writer, "{}  {}: ", indent, k)?;
                    self.write_value(writer, v, 0)?;
                }
                writeln!(writer, "{}}}", indent)?;
            }
        }

        Ok(())
    }
}

impl Default for XoxDocument {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub dependencies: Vec<Dependency>,
    pub dev_dependencies: Vec<Dependency>,
    pub features: Vec<String>,
    pub workspace: Option<WorkspaceConfig>,
    pub nix: Option<NixConfig>,
    pub git: Option<GitConfig>,
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub git: Option<String>,
    pub branch: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceConfig {
    pub members: Vec<String>,
    pub exclude: Vec<String>,
    pub default_members: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct NixConfig {
    pub enable: bool,
    pub shell: String,
    pub flake: bool,
    pub overlays: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GitConfig {
    pub direct: bool,
    pub shallow: bool,
    pub depth: Option<usize>,
    pub single_branch: bool,
}

#[derive(Debug, Clone)]
pub struct LockFile {
    pub version: u32,
    pub packages: HashMap<String, PackageLock>,
}

#[derive(Debug, Clone)]
pub struct PackageLock {
    pub version: String,
    pub integrity: String,
    pub resolved: ResolvedDependency,
}

#[derive(Debug, Clone)]
pub struct ResolvedDependency {
    pub name: String,
    pub version: String,
    pub source: DependencySource,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum DependencySource {
    Registry,
    Git(String),
    Path(String),
    Workspace,
}

pub struct XoxPackageManager {
    config: Package,
    lock_file: Option<LockFile>,
    node_modules_path: PathBuf,
    xox_store_path: PathBuf,
}

impl XoxPackageManager {
    pub fn new() -> XoxResult<Self> {
        let config = Self::load_config()?;
        let lock_file = Self::load_lock_file()?;
        let node_modules_path = PathBuf::from("node_modules");
        let xox_store_path = PathBuf::from(".xox");

        Ok(XoxPackageManager {
            config,
            lock_file,
            node_modules_path,
            xox_store_path,
        })
    }

    fn load_config() -> XoxResult<Package> {
        let mut config_files = Vec::new();

        let config_files_list = vec![
            "package.chim",
            "chim.toon",
            "chim.toml",
            "chim.yaml",
            "chim.json",
            "chim.xml",
            "chim.nix",
        ];

        for config_file in &config_files_list {
            if Path::new(config_file).exists() {
                config_files.push(config_file.to_string());
            }
        }

        if config_files.is_empty() {
            return Err(XoxError::ConfigNotFoundError(
                "No configuration file found. Expected one of: package.chim, chim.toon, chim.toml, chim.yaml, chim.json, chim.xml, chim.nix".to_string()
            ));
        }

        if config_files.len() > 1 {
            return Err(XoxError::MultipleConfigFilesError(format!(
                "Multiple configuration files found: {:?}. Please use only one.",
                config_files
            )));
        }

        let config_file = &config_files[0];
        let content = fs::read_to_string(config_file)
            .map_err(|e| XoxError::ParseError(e.to_string()))?;

        let doc = match config_file.as_str() {
            file if file.ends_with(".toon") || file == "package.chim" => {
                XoxParser::parse_str(&content)?
            }
            file if file.ends_with(".toml") => {
                Self::parse_toml(&content)?
            }
            file if file.ends_with(".yaml") || file.ends_with(".yml") => {
                Self::parse_yaml(&content)?
            }
            file if file.ends_with(".json") => {
                Self::parse_json(&content)?
            }
            file if file.ends_with(".xml") => {
                Self::parse_xml(&content)?
            }
            file if file.ends_with(".nix") => {
                Self::parse_nix(&content)?
            }
            _ => {
                return Err(XoxError::ParseError(format!("Unknown config format: {}", config_file)));
            }
        };

        let mut package = Package {
            name: doc.get_root()
                .get("name")
                .and_then(|v| v.as_string())
                .unwrap_or_default(),
            version: doc.get_root()
                .get("version")
                .and_then(|v| v.as_i64())
                .map(|v| v.to_string())
                .unwrap_or_default(),
            description: doc.get_root()
                .get("description")
                .and_then(|v| v.as_string()),
            dependencies: Self::parse_dependencies(doc.get_root().get("dependencies"))?,
            dev_dependencies: Self::parse_dependencies(doc.get_root().get("devDependencies"))?,
            features: Self::parse_features(doc.get_root().get("features"))?,
            workspace: Self::parse_workspace(doc.get_root().get("workspace"))?,
            nix: Self::parse_nix(doc.get_root().get("nix"))?,
            git: Self::parse_git(doc.get_root().get("git"))?,
        };

        Ok(package)
    }

    fn parse_toml(content: &str) -> XoxResult<XoxDocument> {
        let mut doc = XoxDocument::new();
        let mut root_map = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().trim_matches('"').to_string();

                if let Ok(num) = value.parse::<i64>() {
                    root_map.insert(key, XoxValue::int(num));
                } else if let Ok(float_val) = value.parse::<f64>() {
                    root_map.insert(key, XoxValue::float(float_val));
                } else if value == "true" {
                    root_map.insert(key, XoxValue::bool(true));
                } else if value == "false" {
                    root_map.insert(key, XoxValue::bool(false));
                } else {
                    root_map.insert(key, XoxValue::string(value));
                }
            }
        }

        doc.set_root(XoxValue::map(root_map));
        Ok(doc)
    }

    fn parse_yaml(content: &str) -> XoxResult<XoxDocument> {
        let mut doc = XoxDocument::new();
        let mut root_map = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_string();
                let value = value.trim().trim_matches('"').to_string();

                if let Ok(num) = value.parse::<i64>() {
                    root_map.insert(key, XoxValue::int(num));
                } else if let Ok(float_val) = value.parse::<f64>() {
                    root_map.insert(key, XoxValue::float(float_val));
                } else if value == "true" {
                    root_map.insert(key, XoxValue::bool(true));
                } else if value == "false" {
                    root_map.insert(key, XoxValue::bool(false));
                } else {
                    root_map.insert(key, XoxValue::string(value));
                }
            }
        }

        doc.set_root(XoxValue::map(root_map));
        Ok(doc)
    }

    fn parse_json(content: &str) -> XoxResult<XoxDocument> {
        let mut doc = XoxDocument::new();
        let mut root_map = HashMap::new();

        let content = content.trim();
        if content.starts_with('{') && content.ends_with('}') {
            let inner = &content[1..content.len() - 1];
            for pair in inner.split(',') {
                let pair = pair.trim();
                if let Some((key, value)) = pair.split_once(':') {
                    let key = key.trim().trim_matches('"').to_string();
                    let value = value.trim().trim_matches('"').to_string();

                    if let Ok(num) = value.parse::<i64>() {
                        root_map.insert(key, XoxValue::int(num));
                    } else if let Ok(float_val) = value.parse::<f64>() {
                        root_map.insert(key, XoxValue::float(float_val));
                    } else if value == "true" {
                        root_map.insert(key, XoxValue::bool(true));
                    } else if value == "false" {
                        root_map.insert(key, XoxValue::bool(false));
                    } else {
                        root_map.insert(key, XoxValue::string(value));
                    }
                }
            }
        }

        doc.set_root(XoxValue::map(root_map));
        Ok(doc)
    }

    fn parse_xml(content: &str) -> XoxResult<XoxDocument> {
        let mut doc = XoxDocument::new();
        let mut root_map = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('<') && line.starts_with("<!--") {
                continue;
            }

            if let Some(start_tag) = line.strip_prefix('<').and_then(|s| s.strip_suffix('>')) {
                if start_tag.starts_with("!--") {
                    continue;
                }

                let tag_name = start_tag.trim();
                if tag_name.starts_with('/') {
                    continue;
                }

                if let Some((key, value)) = tag_name.split_once('=') {
                    let key = key.trim().to_string();
                    let value = value.trim().trim_matches('"').to_string();

                    if let Ok(num) = value.parse::<i64>() {
                        root_map.insert(key, XoxValue::int(num));
                    } else if let Ok(float_val) = value.parse::<f64>() {
                        root_map.insert(key, XoxValue::float(float_val));
                    } else if value == "true" {
                        root_map.insert(key, XoxValue::bool(true));
                    } else if value == "false" {
                        root_map.insert(key, XoxValue::bool(false));
                    } else {
                        root_map.insert(key, XoxValue::string(value));
                    }
                }
            }
        }

        doc.set_root(XoxValue::map(root_map));
        Ok(doc)
    }

    fn parse_nix(content: &str) -> XoxResult<XoxDocument> {
        let mut doc = XoxDocument::new();
        let mut root_map = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().trim_matches('"').to_string();

                if let Ok(num) = value.parse::<i64>() {
                    root_map.insert(key, XoxValue::int(num));
                } else if let Ok(float_val) = value.parse::<f64>() {
                    root_map.insert(key, XoxValue::float(float_val));
                } else if value == "true" {
                    root_map.insert(key, XoxValue::bool(true));
                } else if value == "false" {
                    root_map.insert(key, XoxValue::bool(false));
                } else {
                    root_map.insert(key, XoxValue::string(value));
                }
            }
        }

        doc.set_root(XoxValue::map(root_map));
        Ok(doc)
    }

    fn load_lock_file() -> XoxResult<Option<LockFile>> {
        let lock_path = PathBuf::from("xox.lock");
        if !lock_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&lock_path)
            .map_err(|e| XoxError::LockError(e.to_string()))?;

        let doc = XoxParser::parse_str(&content)?;

        let version = doc.get_root()
            .get("version")
            .and_then(|v| v.as_i64())
            .map(|v| v as u32)
            .unwrap_or(1);

        let mut packages = HashMap::new();

        if let Some(packages_map) = doc.get_root().get("packages").and_then(|v| v.as_map()) {
            for (name, lock) in packages_map {
                if let Some(lock_map) = lock.as_map() {
                    let package_lock = PackageLock {
                        version: lock_map.get("version")
                            .and_then(|v| v.as_string())
                            .unwrap_or_default(),
                        integrity: lock_map.get("integrity")
                            .and_then(|v| v.as_string())
                            .unwrap_or_default(),
                        resolved: Self::parse_resolved(lock_map.get("resolved"))?,
                    };
                    packages.insert(name.clone(), package_lock);
                }
            }
        }

        Ok(Some(LockFile { version, packages }))
    }

    fn parse_dependencies(value: Option<&XoxValue>) -> XoxResult<Vec<Dependency>> {
        let mut deps = Vec::new();

        if let Some(deps_map) = value.and_then(|v| v.as_map()) {
            for (name, dep_value) in deps_map {
                if let Some(dep_map) = dep_value.as_map() {
                    let dep = Dependency {
                        name: name.clone(),
                        version: dep_map.get("version")
                            .and_then(|v| v.as_string())
                            .unwrap_or_default(),
                        git: dep_map.get("git")
                            .and_then(|v| v.as_string()),
                        branch: dep_map.get("branch")
                            .and_then(|v| v.as_string()),
                        path: dep_map.get("path")
                            .and_then(|v| v.as_string()),
                    };
                    deps.push(dep);
                }
            }
        }

        Ok(deps)
    }

    fn parse_features(value: Option<&XoxValue>) -> XoxResult<Vec<String>> {
        let mut features = Vec::new();

        if let Some(features_list) = value.and_then(|v| v.as_list()) {
            for feature in features_list {
                if let Some(feature_str) = feature.as_string() {
                    features.push(feature_str.clone());
                }
            }
        }

        Ok(features)
    }

    fn parse_workspace(value: Option<&XoxValue>) -> XoxResult<Option<WorkspaceConfig>> {
        if value.is_none() {
            return Ok(None);
        }

        if let Some(workspace_map) = value.and_then(|v| v.as_map()) {
            let members = workspace_map.get("members")
                .and_then(|v| v.as_list())
                .map(|list| {
                    list.iter()
                        .filter_map(|v| v.as_string().cloned())
                        .collect()
                })
                .unwrap_or_default();

            let exclude = workspace_map.get("exclude")
                .and_then(|v| v.as_list())
                .map(|list| {
                    list.iter()
                        .filter_map(|v| v.as_string().cloned())
                        .collect()
                })
                .unwrap_or_default();

            let default_members = workspace_map.get("defaultMembers")
                .and_then(|v| v.as_list())
                .map(|list| {
                    list.iter()
                        .filter_map(|v| v.as_string().cloned())
                        .collect()
                })
                .unwrap_or_default();

            return Ok(Some(WorkspaceConfig {
                members,
                exclude,
                default_members,
            }));
        }

        Ok(None)
    }

    fn parse_nix(value: Option<&XoxValue>) -> XoxResult<Option<NixConfig>> {
        if value.is_none() {
            return Ok(None);
        }

        if let Some(nix_map) = value.and_then(|v| v.as_map()) {
            let enable = nix_map.get("enable")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let shell = nix_map.get("shell")
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "nix-shell".to_string());

            let flake = nix_map.get("flake")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let overlays = nix_map.get("overlays")
                .and_then(|v| v.as_list())
                .map(|list| {
                    list.iter()
                        .filter_map(|v| v.as_string().cloned())
                        .collect()
                })
                .unwrap_or_default();

            return Ok(Some(NixConfig {
                enable,
                shell,
                flake,
                overlays,
            }));
        }

        Ok(None)
    }

    fn parse_git(value: Option<&XoxValue>) -> XoxResult<Option<GitConfig>> {
        if value.is_none() {
            return Ok(None);
        }

        if let Some(git_map) = value.and_then(|v| v.as_map()) {
            let direct = git_map.get("direct")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let shallow = git_map.get("shallow")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let depth = git_map.get("depth")
                .and_then(|v| v.as_i64())
                .map(|v| v as usize);

            let single_branch = git_map.get("singleBranch")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            return Ok(Some(GitConfig {
                direct,
                shallow,
                depth,
                single_branch,
            }));
        }

        Ok(None)
    }

    fn parse_resolved(value: Option<&XoxValue>) -> XoxResult<ResolvedDependency> {
        if value.is_none() {
            return Err(XoxError::ParseError("Missing resolved dependency".to_string()));
        }

        if let Some(resolved_map) = value.and_then(|v| v.as_map()) {
            let name = resolved_map.get("name")
                .and_then(|v| v.as_string())
                .unwrap_or_default();

            let version = resolved_map.get("version")
                .and_then(|v| v.as_string())
                .unwrap_or_default();

            let source_str = resolved_map.get("source")
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "registry".to_string());

            let source = match source_str.as_str() {
                "registry" => DependencySource::Registry,
                "workspace" => DependencySource::Workspace,
                s if s.starts_with("git:") => {
                    let url = s.strip_prefix("git:").unwrap_or(s);
                    DependencySource::Git(url.to_string())
                }
                s if s.starts_with("path:") => {
                    let path = s.strip_prefix("path:").unwrap_or(s);
                    DependencySource::Path(path.to_string())
                }
                _ => DependencySource::Registry,
            };

            let dependencies = resolved_map.get("dependencies")
                .and_then(|v| v.as_list())
                .map(|list| {
                    list.iter()
                        .filter_map(|v| v.as_string().cloned())
                        .collect()
                })
                .unwrap_or_default();

            return Ok(ResolvedDependency {
                name,
                version,
                source,
                dependencies,
            });
        }

        Err(XoxError::ParseError("Invalid resolved dependency".to_string()))
    }

    pub fn install(&self, package: &str) -> XoxResult<()> {
        if let Some(lock) = &self.lock_file {
            if lock.packages.contains_key(package) {
                return self.install_from_lock(package);
            }
        }

        self.install_from_registry(package)
    }

    fn install_from_lock(&self, package: &str) -> XoxResult<()> {
        let lock = self.lock_file.as_ref()
            .ok_or_else(|| XoxError::LockError("No lock file".to_string()))?;

        let package_lock = lock.packages.get(package)
            .ok_or_else(|| XoxError::LockError(format!("Package {} not in lock file", package)))?;

        self.create_hard_links(package, &package_lock.version)?;
        Ok(())
    }

    fn install_from_registry(&self, package: &str) -> XoxResult<()> {
        println!("Installing {} from registry...", package);
        Ok(())
    }

    fn create_hard_links(&self, package: &str, version: &str) -> Self {
        let package_path = self.node_modules_path.join(".xox").join(package).join(version);
        let target_path = self.node_modules_path.join(package);

        fs::create_dir_all(&package_path)
            .map_err(|e| XoxError::LinkError(e.to_string()))?;

        if target_path.exists() {
            fs::remove_file(&target_path)
                .map_err(|e| XoxError::LinkError(e.to_string()))?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(&package_path, &target_path)
                .map_err(|e| XoxError::LinkError(e.to_string()))?;
        }

        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_file;
            symlink_file(&package_path, &target_path)
                .map_err(|e| XoxError::LinkError(e.to_string()))?;
        }

        println!("Created hard link for {}@{}", package, version);
    }

    pub fn update_lock_file(&mut self) -> XoxResult<()> {
        let mut packages = HashMap::new();

        for dep in &self.config.dependencies {
            let resolved = ResolvedDependency {
                name: dep.name.clone(),
                version: dep.version.clone(),
                source: if dep.git.is_some() {
                    DependencySource::Git(dep.git.clone().unwrap_or_default())
                } else if dep.path.is_some() {
                    DependencySource::Path(dep.path.clone().unwrap_or_default())
                } else {
                    DependencySource::Registry
                },
                dependencies: Vec::new(),
            };

            let package_lock = PackageLock {
                version: dep.version.clone(),
                integrity: format!("sha256-{}", Self::calculate_integrity(&dep)),
                resolved,
            };

            packages.insert(dep.name.clone(), package_lock);
        }

        let mut lock_doc = XoxDocument::new();
        let mut lock_root = HashMap::new();

        let mut packages_map = HashMap::new();
        for (name, lock) in &packages {
            let mut lock_map = HashMap::new();
            lock_map.insert("version".to_string(), XoxValue::string(lock.version.clone()));
            lock_map.insert("integrity".to_string(), XoxValue::string(lock.integrity.clone()));

            let mut resolved_map = HashMap::new();
            resolved_map.insert("name".to_string(), XoxValue::string(lock.resolved.name.clone()));
            resolved_map.insert("version".to_string(), XoxValue::string(lock.resolved.version.clone()));

            let mut source_str = match &lock.resolved.source {
                DependencySource::Registry => "registry".to_string(),
                DependencySource::Git(url) => format!("git:{}", url),
                DependencySource::Path(path) => format!("path:{}", path),
                DependencySource::Workspace => "workspace".to_string(),
            };
            resolved_map.insert("source".to_string(), XoxValue::string(source_str));

            let deps_list = lock.resolved.dependencies
                .iter()
                .map(|d| XoxValue::string(d.clone()))
                .collect();
            resolved_map.insert("dependencies".to_string(), XoxValue::list(deps_list));

            lock_map.insert("resolved".to_string(), XoxValue::map(resolved_map));
            packages_map.insert(name.clone(), XoxValue::map(lock_map));
        }

        lock_root.insert("version".to_string(), XoxValue::int(1));
        lock_root.insert("packages".to_string(), XoxValue::map(packages_map));

        lock_doc.set_root(XoxValue::map(lock_root));

        let lock_content = lock_doc.to_string()?;
        fs::write("xox.lock", lock_content)
            .map_err(|e| XoxError::LockError(e.to_string()))?;

        self.lock_file = Some(LockFile {
            version: 1,
            packages,
        });

        Ok(())
    }

    fn calculate_integrity(dep: &Dependency) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;

        let mut hasher = DefaultHasher::new();
        format!("{}@{}", dep.name, dep.version).hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    pub fn generate_cargo_toml(&self) -> XoxResult<String> {
        let mut toml = String::new();

        toml.push_str("[package]\n");
        toml.push_str(&format!("name = \"{}\"\n", self.config.name));
        toml.push_str(&format!("version = \"{}\"\n", self.config.version));

        if let Some(ref desc) = self.config.description {
            toml.push_str(&format!("description = \"{}\"\n", desc));
        }

        toml.push_str("\n[dependencies]\n");
        for dep in &self.config.dependencies {
            if let Some(ref git) = dep.git {
                toml.push_str(&format!("{} = {{ git = \"{}\"", dep.name, git));
                if let Some(ref branch) = dep.branch {
                    toml.push_str(&format!(", branch = \"{}\"", branch));
                }
                toml.push_str(" }\n");
            } else if let Some(ref path) = dep.path {
                toml.push_str(&format!("{} = {{ path = \"{}\" }}\n", dep.name, path));
            } else {
                toml.push_str(&format!("{} = \"{}\"\n", dep.name, dep.version));
            }
        }

        toml.push_str("\n[workspace]\n");
        if let Some(ref workspace) = self.config.workspace {
            toml.push_str("members = [");
            for (i, member) in workspace.members.iter().enumerate() {
                if i > 0 {
                    toml.push_str(", ");
                }
                toml.push_str(&format!("\"{}\"", member));
            }
            toml.push_str("]\n");

            if !workspace.exclude.is_empty() {
                toml.push_str("exclude = [");
                for (i, excl) in workspace.exclude.iter().enumerate() {
                    if i > 0 {
                        toml.push_str(", ");
                    }
                    toml.push_str(&format!("\"{}\"", excl));
                }
                toml.push_str("]\n");
            }
        }

        Ok(toml)
    }

    pub fn generate_nix_flake(&self) -> XoxResult<String> {
        let mut flake = String::new();

        flake.push_str("{\n");
        flake.push_str("  description = \"Chim Package\";\n");
        flake.push_str("  inputs = {\n");
        flake.push_str("    nixpkgs.url = \"github:NixOS/nixpkgs/nixos-unstable\";\n");
        flake.push_str("    rust-overlay.url = \"github:oxalica/rust-overlay\";\n");
        flake.push_str("  };\n");
        flake.push_str("  outputs = { self, ...inputs }:\n");
        flake.push_str("    let pkgs = import nixpkgs { system = \"x86_64-linux\"; };\n");
        flake.push_str("    in {\n");
        flake.push_str(&format!("      packages.x86_64-linux.{} = pkgs.rustPlatform.buildRustPackage {{\n", self.config.name));
        flake.push_str(&format!("        pname = \"{}\";\n", self.config.name));
        flake.push_str(&format!("        version = \"{}\";\n", self.config.version));
        flake.push_str("        src = ./;\n");
        flake.push_str("        cargoLock = ./Cargo.lock;\n");
        flake.push_str("        cargoToml = ./Cargo.toml;\n");
        flake.push_str("        buildInputs = with pkgs; [ pkgs.openssl pkgs.pkg-config ];\n");
        flake.push_str("        nativeBuildInputs = with pkgs; [ pkgs.cargo pkgs.rustc ];\n");
        flake.push_str("      }};\n");
        flake.push_str("    };\n");
        flake.push_str("  }\n");
        flake.push_str("}\n");

        Ok(flake)
    }

    pub fn git_direct_clone(&self, repo: &str, package: &str) -> XoxResult<()> {
        let url = if repo.starts_with("git+") {
            repo.to_string()
        } else {
            format!("git+https://{}", repo)
        };

        let mut cmd = std::process::Command::new("git");
        cmd.args(["clone", "--depth", "1", &url, &format!(".xox/{}", package)]);

        let output = cmd.output()
            .map_err(|e| XoxError::GitError(e.to_string()))?;

        if !output.status.success() {
            return Err(XoxError::GitError(format!("Git clone failed: {:?}", output)));
        }

        println!("Git direct clone of {} completed", package);
        Ok(())
    }
}

pub struct XoxParser;

impl XoxParser {
    pub fn new() -> Self {
        XoxParser
    }

    pub fn parse<R: Read>(&self, reader: &mut R) -> XoxResult<XoxDocument> {
        let mut content = String::new();
        reader.read_to_string(&mut content)
            .map_err(|e| XoxError::ParseError(e.to_string()))?;

        self.parse_str(&content)
    }

    pub fn parse_str(&self, content: &str) -> XoxResult<XoxDocument> {
        let mut parser = Parser {
            input: content,
            pos: 0,
            metadata: HashMap::new(),
        };
        let root = parser.parse_value()?;
        Ok(XoxDocument {
            root,
            metadata: parser.metadata,
        })
    }
}

impl Default for XoxParser {
    fn default() -> Self {
        Self::new()
    }
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
    metadata: HashMap<String, String>,
}

impl<'a> Parser<'a> {
    fn parse_value(&mut self) -> XoxResult<XoxValue> {
        self.skip_whitespace();

        if self.pos >= self.input.len() {
            return Ok(XoxValue::Null);
        }

        let c = self.input.as_bytes()[self.pos];
        match c {
            b'n' => self.parse_null(),
            b't' => self.parse_true(),
            b'f' => self.parse_false(),
            b'i' => self.parse_integer(),
            b'f' => self.parse_float(),
            b's' => self.parse_string(),
            b'b' => self.parse_binary(),
            b'[' => self.parse_list(),
            b'{' => self.parse_map(),
            b'#' => self.parse_comment(),
            _ => Err(XoxError::ParseError(format!("Unexpected character: {}", c as char))),
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos];
            if c.is_ascii_whitespace() || c == b'\n' || c == b'\r' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn skip_to_next_line(&mut self) {
        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos];
            if c == b'\n' || c == b'\r' {
                self.pos += 1;
                break;
            }
            self.pos += 1;
        }
    }

    fn parse_null(&mut self) -> XoxResult<XoxValue> {
        self.pos += 1;
        Ok(XoxValue::Null)
    }

    fn parse_true(&mut self) -> XoxResult<XoxValue> {
        if self.input[self.pos..].starts_with("true") {
            self.pos += 4;
            Ok(XoxValue::Bool(true))
        } else {
            Err(XoxError::ParseError("Expected 'true'".to_string()))
        }
    }

    fn parse_false(&mut self) -> XoxResult<XoxValue> {
        if self.input[self.pos..].starts_with("false") {
            self.pos += 5;
            Ok(XoxValue::Bool(false))
        } else {
            Err(XoxError::ParseError("Expected 'false'".to_string()))
        }
    }

    fn parse_integer(&mut self) -> XoxResult<XoxValue> {
        self.pos += 1;

        if self.pos >= self.input.len() || self.input.as_bytes()[self.pos] != b':' {
            return Err(XoxError::ParseError("Expected ':' after 'i'".to_string()));
        }
        self.pos += 1;

        let start = self.pos;

        if self.pos < self.input.len() && self.input.as_bytes()[self.pos] == b'-' {
            self.pos += 1;
        }

        while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_digit() {
            self.pos += 1;
        }

        let num_str = &self.input[start..self.pos];
        num_str.parse::<i64>()
            .map(XoxValue::int)
            .map_err(|_| XoxError::ParseError("Invalid integer".to_string()))
    }

    fn parse_float(&mut self) -> XoxResult<XoxValue> {
        self.pos += 1;

        if self.pos >= self.input.len() || self.input.as_bytes()[self.pos] != b':' {
            return Err(XoxError::ParseError("Expected ':' after 'f'".to_string()));
        }
        self.pos += 1;

        let start = self.pos;

        if self.pos < self.input.len() && self.input.as_bytes()[self.pos] == b'-' {
            self.pos += 1;
        }

        while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_digit() {
            self.pos += 1;
        }

        if self.pos < self.input.len() && self.input.as_bytes()[self.pos] == b'.' {
            self.pos += 1;
            while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }

        if self.pos < self.input.len() && (self.input.as_bytes()[self.pos] == b'e' || self.input.as_bytes()[self.pos] == b'E') {
            self.pos += 1;
            if self.pos < self.input.len() && (self.input.as_bytes()[self.pos] == b'+' || self.input.as_bytes()[self.pos] == b'-') {
                self.pos += 1;
            }
            while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }

        let num_str = &self.input[start..self.pos];
        num_str.parse::<f64>()
            .map(XoxValue::float)
            .map_err(|_| XoxError::ParseError("Invalid float".to_string()))
    }

    fn parse_string(&mut self) -> XoxResult<XoxValue> {
        self.pos += 1;

        if self.pos >= self.input.len() {
            return Err(XoxError::ParseError("Unexpected end of input".to_string()));
        }

        let c = self.input.as_bytes()[self.pos];
        self.pos += 1;

        match c {
            b'"' => self.parse_quoted_string(),
            b'|' => self.parse_block_string(),
            _ => Err(XoxError::ParseError("Expected '\"' or '|' after 's'".to_string())),
        }
    }

    fn parse_quoted_string(&mut self) -> XoxResult<XoxValue> {
        let mut result = String::new();

        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos];
            self.pos += 1;

            if c == b'"' {
                return Ok(XoxValue::String(result));
            }

            if c == b'\\' {
                if self.pos >= self.input.len() {
                    return Err(XoxError::ParseError("Unclosed escape sequence".to_string()));
                }
                let escaped = self.input.as_bytes()[self.pos];
                self.pos += 1;
                let decoded = match escaped {
                    b'"' => '"',
                    b'\\' => '\\',
                    b'n' => '\n',
                    b'r' => '\r',
                    b't' => '\t',
                    _ => return Err(XoxError::ParseError("Invalid escape sequence".to_string())),
                };
                result.push(decoded);
            } else {
                result.push(c as char);
            }
        }

        Err(XoxError::ParseError("Unclosed string".to_string()))
    }

    fn parse_block_string(&mut self) -> XoxResult<XoxValue> {
        let mut result = String::new();

        loop {
            if self.pos >= self.input.len() {
                return Err(XoxError::ParseError("Unclosed block string".to_string()));
            }

            let c = self.input.as_bytes()[self.pos];

            if c.is_ascii_whitespace() || c == b'\n' || c == b'\r' {
                self.pos += 1;
                continue;
            }

            let next_c = if self.pos + 1 < self.input.len() {
                self.input.as_bytes()[self.pos + 1]
            } else {
                0
            };

            if next_c.is_ascii_whitespace() || next_c == b'\n' || next_c == b'\r' {
                self.pos += 1;
                break;
            }

            result.push(c as char);
            self.pos += 1;
        }

        Ok(XoxValue::String(result))
    }

    fn parse_binary(&mut self) -> XoxResult<XoxValue> {
        self.pos += 1;

        if self.pos >= self.input.len() || self.input.as_bytes()[self.pos] != b':' {
            return Err(XoxError::ParseError("Expected ':' after 'b'".to_string()));
        }
        self.pos += 1;

        let start = self.pos;

        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos];
            if c.is_ascii_whitespace() || c == b'\n' || c == b'\r' {
                break;
            }
            self.pos += 1;
        }

        let encoded = &self.input[start..self.pos];
        base64::decode(encoded)
            .map(XoxValue::binary)
            .map_err(|_| XoxError::ParseError("Invalid base64 encoding".to_string()))
    }

    fn parse_list(&mut self) -> XoxResult<XoxValue> {
        self.pos += 1;
        self.skip_whitespace();

        let mut list = Vec::new();

        loop {
            self.skip_whitespace();

            if self.pos >= self.input.len() {
                return Err(XoxError::ParseError("Unclosed list".to_string()));
            }

            if self.input.as_bytes()[self.pos] == b']' {
                self.pos += 1;
                break;
            }

            list.push(self.parse_value()?);
        }

        Ok(XoxValue::List(list))
    }

    fn parse_map(&mut self) -> XoxResult<XoxValue> {
        self.pos += 1;
        self.skip_whitespace();

        let mut map = HashMap::new();

        loop {
            self.skip_whitespace();

            if self.pos >= self.input.len() {
                return Err(XoxError::ParseError("Unclosed map".to_string()));
            }

            if self.input.as_bytes()[self.pos] == b'}' {
                self.pos += 1;
                break;
            }

            let key = match self.parse_value()? {
                XoxValue::String(s) => s,
                _ => return Err(XoxError::ParseError("Map key must be a string".to_string())),
            };

            self.skip_whitespace();

            if self.pos >= self.input.len() || self.input.as_bytes()[self.pos] != b':' {
                return Err(XoxError::ParseError("Expected ':' after key".to_string()));
            }
            self.pos += 1;

            self.skip_whitespace();
            let value = self.parse_value()?;
            map.insert(key, value);
        }

        Ok(XoxValue::Map(map))
    }

    fn parse_comment(&mut self) -> XoxResult<XoxValue> {
        self.pos += 1;
        self.skip_whitespace();

        if self.pos >= self.input.len() {
            return Ok(XoxValue::Null);
        }

        let c = self.input.as_bytes()[self.pos];
        self.pos += 1;

        if c == b' ' {
            let start = self.pos;

            while self.pos < self.input.len() {
                let c = self.input.as_bytes()[self.pos];
                if c == b':' {
                    self.pos += 1;
                    let key = self.input[start..self.pos - 1].to_string();
                    let value_start = self.pos;
                    self.skip_to_next_line();
                    let value = self.input[value_start..self.pos].trim().to_string();
                    self.metadata.insert(key, value);
                    break;
                }
                self.pos += 1;
            }
        } else {
            self.skip_to_next_line();
        }

        self.parse_value()
    }
}

pub fn parse<R: Read>(reader: &mut R) -> XoxResult<XoxDocument> {
    XoxParser::new().parse(reader)
}

pub fn parse_str(content: &str) -> XoxResult<XoxDocument> {
    XoxParser::new().parse_str(content)
}

pub fn to_string(value: &XoxValue) -> XoxResult<String> {
    let doc = XoxDocument { root: value.clone(), metadata: HashMap::new() };
    doc.to_string()
}

mod base64 {
    pub fn encode(data: &[u8]) -> String {
        const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        let mut result = String::new();
        let mut i = 0;

        while i < data.len() {
            let chunk = &data[i..std::cmp::min(i + 3, data.len())];
            let mut acc = 0u32;
            let mut bits = 0;

            for &byte in chunk {
                acc = (acc << 8) | (byte as u32);
                bits += 8;
            }

            while bits > 0 {
                let idx = ((acc >> (bits - 6)) & 0x3F) as usize;
                result.push(TABLE[idx] as char);
                bits -= 6;
            }

            i += 3;
        }

        while result.len() % 4 != 0 {
            result.push('=');
        }

        result
    }

    pub fn decode(encoded: &str) -> Result<Vec<u8>, String> {
        const TABLE: &[i8; 128] = &[
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            62, -1, -1, -1, 63, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
            17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
            32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
            48, 49, 50, 51,
        ];

        let encoded = encoded.trim_end_matches('=');
        let mut result = Vec::new();
        let mut i = 0;

        while i < encoded.len() {
            let chunk = &encoded.as_bytes()[i..std::cmp::min(i + 4, encoded.len())];
            let mut acc = 0u32;
            let mut bits = 0;

            for &byte in chunk {
                if byte >= 128 || TABLE[byte as usize] == -1 {
                    return Err("Invalid base64 character".to_string());
                }
                acc = (acc << 6) | (TABLE[byte as usize] as u32);
                bits += 6;
            }

            while bits >= 8 {
                result.push(((acc >> (bits - 8)) & 0xFF) as u8);
                bits -= 8;
            }

            i += 4;
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xox_value() {
        let value = XoxValue::string("hello".to_string());
        assert_eq!(value.as_string(), Some(&"hello".to_string()));
    }

    #[test]
    fn test_xox_parser() {
        let xox = "name: s:\"test\"\nversion: i:100";
        let doc = parse_str(xox).unwrap();
        let root = doc.get_root();
        assert_eq!(root.get("name").and_then(|v| v.as_string()), Some(&"test".to_string()));
        assert_eq!(root.get("version").and_then(|v| v.as_i64()), Some(100));
    }

    #[test]
    fn test_package_manager() {
        let pm = XoxPackageManager::new().unwrap();
        assert!(!pm.config.name.is_empty());
    }

    #[test]
    fn test_cargo_toml_generation() {
        let pm = XoxPackageManager::new().unwrap();
        let toml = pm.generate_cargo_toml().unwrap();
        assert!(toml.contains("[package]"));
        assert!(toml.contains("[dependencies]"));
    }

    #[test]
    fn test_nix_flake_generation() {
        let pm = XoxPackageManager::new().unwrap();
        let flake = pm.generate_nix_flake().unwrap();
        assert!(flake.contains("nixpkgs"));
        assert!(flake.contains("rust-overlay"));
    }

    #[test]
    fn test_base64_encode() {
        let data = vec![1u8, 2u8, 3u8];
        let encoded = base64::encode(&data);
        assert!(!encoded.is_empty());
    }

    #[test]
    fn test_base64_decode() {
        let encoded = "AQID";
        let decoded = base64::decode(encoded).unwrap();
        assert_eq!(decoded, vec![1u8, 2u8, 3u8]);
    }
}
