use crate::types::{Escrow, User};
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

pub struct MemoryStorage {
    escrows: RwLock<HashMap<Uuid, Escrow>>,
    users: RwLock<HashMap<Uuid, User>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            escrows: RwLock::new(HashMap::new()),
            users: RwLock::new(HashMap::new()),
        }
    }
    
    pub fn create_escrow(&self, escrow: Escrow) -> Result<(), String> {
        let mut escrows = self.escrows.write()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        escrows.insert(escrow.id, escrow);
        Ok(())
    }
    
    pub fn get_escrow(&self, id: Uuid) -> Result<Option<Escrow>, String> {
        let escrows = self.escrows.read()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        Ok(escrows.get(&id).cloned())
    }
    
    pub fn update_escrow(&self, escrow: Escrow) -> Result<(), String> {
        let mut escrows = self.escrows.write()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        escrows.insert(escrow.id, escrow);
        Ok(())
    }
    
    pub fn get_user(&self, id: Uuid) -> Result<Option<User>, String> {
        let users = self.users.read()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        Ok(users.get(&id).cloned())
    }
    
    pub fn create_user(&self, user: User) -> Result<(), String> {
        let mut users = self.users.write()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        users.insert(user.id, user);
        Ok(())
    }
    
    pub fn list_escrows(&self) -> Result<Vec<Escrow>, String> {
        let escrows = self.escrows.read()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        Ok(escrows.values().cloned().collect())
    }
}