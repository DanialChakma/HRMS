#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Role {
    Admin = 1,
    Hr = 2,
    Employee = 3,
    System=4,
    ApiUser=5
}

impl Role {
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            1 => Some(Role::Admin),
            2 => Some(Role::Hr),
            3 => Some(Role::Employee),
            4 => Some(Role::System),
            5 => Some(Role::ApiUser),
            _ => None,
        }
    }
}
