#[derive(Debug, Clone)]
pub struct EmployeeInfo {
    pub id: u32,
    pub name: String,
    pub holiday_allowance: u32,
    pub department_id: u32,
}

impl EmployeeInfo {
    pub fn new(
        id: u32,
        name: String,
        holiday_allowance: u32,
        department_id: u32,
    ) -> Self {
        Self {
            id,
            name,
            holiday_allowance,
            department_id,
        }
    }
}

