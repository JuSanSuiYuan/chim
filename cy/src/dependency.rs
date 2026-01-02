use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use crate::config::{PackageConfig, LockFile, LockedPackage};

/// 依赖解析器
pub struct DependencyResolver {
    /// 已解析的包
    resolved: HashMap<String, String>,
    /// 待解析的包
    pending: Vec<(String, String)>,
}

impl DependencyResolver {
    pub fn new() -> Self {
        Self {
            resolved: HashMap::new(),
            pending: Vec::new(),
        }
    }

    /// 解析依赖树
    pub async fn resolve(&mut self, config: &PackageConfig) -> Result<HashMap<String, String>> {
        // 添加直接依赖到待解析列表
        for (name, version) in &config.dependencies {
            self.pending.push((name.clone(), version.clone()));
        }
        
        for (name, version) in &config.dev_dependencies {
            self.pending.push((name.clone(), version.clone()));
        }

        // 解析所有依赖（包括传递依赖）
        while let Some((name, version)) = self.pending.pop() {
            if self.resolved.contains_key(&name) {
                continue;
            }

            // 解析版本
            let resolved_version = self.resolve_version(&name, &version).await?;
            self.resolved.insert(name.clone(), resolved_version.clone());

            // 获取该包的依赖
            let deps = self.fetch_dependencies(&name, &resolved_version).await?;
            for (dep_name, dep_version) in deps {
                if !self.resolved.contains_key(&dep_name) {
                    self.pending.push((dep_name, dep_version));
                }
            }
        }

        Ok(self.resolved.clone())
    }

    /// 解析版本号（支持语义化版本）
    async fn resolve_version(&self, name: &str, version: &str) -> Result<String> {
        // 简化实现：直接返回版本号
        // 实际实现需要支持 ^1.0.0, ~1.0.0, >=1.0.0 等语义化版本规则
        Ok(version.trim_start_matches('^').trim_start_matches('~').to_string())
    }

    /// 获取包的依赖列表
    async fn fetch_dependencies(&self, name: &str, version: &str) -> Result<HashMap<String, String>> {
        // TODO: 从注册表获取包的依赖信息
        // 目前返回空依赖列表
        Ok(HashMap::new())
    }
}

/// 依赖图（用于检测循环依赖）
pub struct DependencyGraph {
    /// 邻接表表示的图
    graph: HashMap<String, HashSet<String>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            graph: HashMap::new(),
        }
    }

    /// 添加依赖边
    pub fn add_edge(&mut self, from: String, to: String) {
        self.graph.entry(from).or_insert_with(HashSet::new).insert(to);
    }

    /// 检测循环依赖
    pub fn detect_cycle(&self) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for node in self.graph.keys() {
            if self.dfs_cycle(node, &mut visited, &mut rec_stack, &mut path) {
                return Some(path);
            }
        }

        None
    }

    fn dfs_cycle(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> bool {
        if rec_stack.contains(node) {
            path.push(node.to_string());
            return true;
        }

        if visited.contains(node) {
            return false;
        }

        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(neighbors) = self.graph.get(node) {
            for neighbor in neighbors {
                if self.dfs_cycle(neighbor, visited, rec_stack, path) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        path.pop();
        false
    }

    /// 拓扑排序（确定安装顺序）
    pub fn topological_sort(&self) -> Result<Vec<String>> {
        let mut in_degree = HashMap::new();
        let mut queue = Vec::new();
        let mut result = Vec::new();

        // 计算入度
        for node in self.graph.keys() {
            in_degree.entry(node.clone()).or_insert(0);
        }

        for neighbors in self.graph.values() {
            for neighbor in neighbors {
                *in_degree.entry(neighbor.clone()).or_insert(0) += 1;
            }
        }

        // 找到所有入度为0的节点
        for (node, &degree) in &in_degree {
            if degree == 0 {
                queue.push(node.clone());
            }
        }

        // 拓扑排序
        while let Some(node) = queue.pop() {
            result.push(node.clone());

            if let Some(neighbors) = self.graph.get(&node) {
                for neighbor in neighbors {
                    let degree = in_degree.get_mut(neighbor).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push(neighbor.clone());
                    }
                }
            }
        }

        // 检查是否有循环
        if result.len() != in_degree.len() {
            anyhow::bail!("检测到循环依赖");
        }

        Ok(result)
    }
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}
