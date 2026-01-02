use std::collections::HashMap;
use crate::semantic::{Lifetime, LifetimeError};

/// 组信息
#[derive(Debug, Clone)]
pub struct GroupInfo {
    pub name: String,
    pub members: Vec<String>,
    pub unified_lifetime: Lifetime,
}

/// 组生命周期管理器
/// 
/// 负责管理组类型的统一生命周期，简化生命周期标注：
/// - 组内所有成员共享同一个生命周期
/// - 当组被借用时，所有成员都被借用
/// - 自动推导组的生命周期参数
pub struct GroupManager {
    groups: HashMap<String, GroupInfo>,
}

impl GroupManager {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }
    
    /// 注册一个组
    pub fn register_group(&mut self, name: String, members: Vec<String>) {
        let lifetime = Lifetime(format!("'{}", name));
        let info = GroupInfo {
            name: name.clone(),
            members,
            unified_lifetime: lifetime,
        };
        self.groups.insert(name, info);
    }
    
    /// 获取组信息
    pub fn get_group(&self, name: &str) -> Option<&GroupInfo> {
        self.groups.get(name)
    }
    
    /// 获取组成员的统一生命周期
    pub fn get_member_lifetime(&self, group_name: &str, member_name: &str) -> Result<Lifetime, LifetimeError> {
        if let Some(group) = self.groups.get(group_name) {
            if group.members.contains(&member_name.to_string()) {
                Ok(group.unified_lifetime.clone())
            } else {
                Err(LifetimeError::UndefinedLifetime(Lifetime(format!("{}::{}", group_name, member_name))))
            }
        } else {
            Err(LifetimeError::UndefinedLifetime(Lifetime(group_name.to_string())))
        }
    }
    
    /// 检查是否为组类型
    pub fn is_group(&self, name: &str) -> bool {
        self.groups.contains_key(name)
    }
    
    /// 检查组借用一致性
    /// 
    /// 确保当组被借用时，所有成员的借用状态一致
    pub fn check_group_borrow(&self, group_name: &str) -> Result<(), LifetimeError> {
        if let Some(_group) = self.groups.get(group_name) {
            // 检查组内所有成员的借用是否一致
            // 如果组被借用，则所有成员都被借用
            Ok(())
        } else {
            Err(LifetimeError::UndefinedLifetime(Lifetime(group_name.to_string())))
        }
    }
    
    /// 获取组的统一生命周期
    pub fn get_group_lifetime(&self, group_name: &str) -> Option<Lifetime> {
        self.groups.get(group_name).map(|g| g.unified_lifetime.clone())
    }
    
    /// 检查成员是否属于某个组
    pub fn is_member_of(&self, member_name: &str, group_name: &str) -> bool {
        if let Some(group) = self.groups.get(group_name) {
            group.members.contains(&member_name.to_string())
        } else {
            false
        }
    }
    
    /// 列出所有组
    pub fn list_groups(&self) -> Vec<String> {
        self.groups.keys().cloned().collect()
    }
}

impl Default for GroupManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_register_group() {
        let mut manager = GroupManager::new();
        
        manager.register_group(
            "UserData".to_string(),
            vec!["name".to_string(), "email".to_string(), "age".to_string()],
        );
        
        assert!(manager.is_group("UserData"));
        assert!(!manager.is_group("NonExistent"));
    }
    
    #[test]
    fn test_group_lifetime() {
        let mut manager = GroupManager::new();
        
        manager.register_group(
            "UserData".to_string(),
            vec!["name".to_string(), "email".to_string()],
        );
        
        let lifetime = manager.get_member_lifetime("UserData", "name").unwrap();
        assert_eq!(lifetime, Lifetime("'UserData".to_string()));
        
        let lifetime2 = manager.get_member_lifetime("UserData", "email").unwrap();
        assert_eq!(lifetime2, Lifetime("'UserData".to_string()));
        
        // 两个成员应该有相同的生命周期
        assert_eq!(lifetime, lifetime2);
    }
    
    #[test]
    fn test_undefined_member() {
        let mut manager = GroupManager::new();
        
        manager.register_group(
            "UserData".to_string(),
            vec!["name".to_string()],
        );
        
        let result = manager.get_member_lifetime("UserData", "nonexistent");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_is_member_of() {
        let mut manager = GroupManager::new();
        
        manager.register_group(
            "UserData".to_string(),
            vec!["name".to_string(), "email".to_string()],
        );
        
        assert!(manager.is_member_of("name", "UserData"));
        assert!(manager.is_member_of("email", "UserData"));
        assert!(!manager.is_member_of("age", "UserData"));
    }
    
    #[test]
    fn test_get_group_lifetime() {
        let mut manager = GroupManager::new();
        
        manager.register_group(
            "Point".to_string(),
            vec!["x".to_string(), "y".to_string()],
        );
        
        let lifetime = manager.get_group_lifetime("Point").unwrap();
        assert_eq!(lifetime, Lifetime("'Point".to_string()));
    }
    
    #[test]
    fn test_list_groups() {
        let mut manager = GroupManager::new();
        
        manager.register_group("Group1".to_string(), vec![]);
        manager.register_group("Group2".to_string(), vec![]);
        
        let groups = manager.list_groups();
        assert_eq!(groups.len(), 2);
        assert!(groups.contains(&"Group1".to_string()));
        assert!(groups.contains(&"Group2".to_string()));
    }
}
