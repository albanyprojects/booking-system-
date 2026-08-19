
use crate::booking::{
    ApprovalStatus,
    Booking,
    BookingEvaluation,
    RuleResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagerRole {
    TradingManager,
    LogisticsManager,
    Director,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagerAction {
    View,
    Approve,
    Reject,
    OverrideApproval,
    OverrideRules,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalOverride {
    None,
    ForceApproved,
    ForceRejected,
    RuleOverride,
}

#[derive(Debug, Clone)]
pub struct ManagerAccount {
    pub user_id: u32,
    pub role: ManagerRole,

    // Department IDs loaded from the database.
    pub managed_departments: Vec<u32>,

    // Permissions loaded from the database.
    pub can_override_approval: bool,
    pub can_override_rules: bool,
}

#[derive(Debug, Clone)]
pub struct ApprovalDecision {
    pub booking_id: u32,
    pub manager_user_id: u32,
    pub approval_status: ApprovalStatus,
    pub rule_result: RuleResult,
    pub override_type: ApprovalOverride,
    pub reason: String,
}

#[derive(Debug)]
pub enum ManagerError {
    NotAuthorised,
    DepartmentNotManaged,
    ApprovalNotPermitted,
    OverrideNotPermitted,
    EmptyReason,
}

pub fn can_manage_department(
    manager: &ManagerAccount,
    department_id: u32,
) -> bool {
    manager
        .managed_departments
        .contains(&department_id)
}

pub fn can_view_booking(
    manager: &ManagerAccount,
    booking: &Booking,
) -> bool {
    can_manage_department(
        manager,
        booking.department_id,
    )
}

pub fn can_perform_action(
    manager: &ManagerAccount,
    booking: &Booking,
    action: ManagerAction,
) -> bool {
    if !can_view_booking(manager, booking) {
        return false;
    }

    match action {
        ManagerAction::View => true,

        ManagerAction::Approve |
        ManagerAction::Reject => true,

        ManagerAction::OverrideApproval => {
            manager.can_override_approval
        }

        ManagerAction::OverrideRules => {
            manager.can_override_rules
        }
    }
}

pub fn visible_departments(
    manager: &ManagerAccount,
) -> &[u32] {
    &manager.managed_departments
}

pub fn approve(
    manager: &ManagerAccount,
    booking: &Booking,
    evaluation: &BookingEvaluation,
    reason: String,
) -> Result<ApprovalDecision, ManagerError> {
    if !can_perform_action(
        manager,
        booking,
        ManagerAction::Approve,
    ) {
        return Err(
            ManagerError::ApprovalNotPermitted
        );
    }

    if reason.trim().is_empty() {
        return Err(
            ManagerError::EmptyReason
        );
    }

    Ok(ApprovalDecision {
        booking_id: booking.id,
        manager_user_id: manager.user_id,
        approval_status: ApprovalStatus::Approved,
        rule_result: evaluation.result,
        override_type: ApprovalOverride::None,
        reason,
    })
}

pub fn reject(
    manager: &ManagerAccount,
    booking: &Booking,
    evaluation: &BookingEvaluation,
    reason: String,
) -> Result<ApprovalDecision, ManagerError> {
    if !can_perform_action(
        manager,
        booking,
        ManagerAction::Reject,
    ) {
        return Err(
            ManagerError::ApprovalNotPermitted
        );
    }
 
    if reason.trim().is_empty() {
        return Err(
            ManagerError::EmptyReason
        );
    }

    Ok(ApprovalDecision {
        booking_id: booking.id,
        manager_user_id: manager.user_id,
        approval_status: ApprovalStatus::Rejected,
        rule_result: evaluation.result,
        override_type: ApprovalOverride::None,
        reason,
    })
}

pub fn override_approval(
    manager: &ManagerAccount,
    booking: &Booking,
    approval: ApprovalStatus,
    evaluation: &BookingEvaluation,
    reason: String,
) -> Result<ApprovalDecision, ManagerError> {
    if !can_perform_action(
        manager,
        booking,
        ManagerAction::OverrideApproval,
    ) {
        return Err(
            ManagerError::OverrideNotPermitted
        );
    }

    if reason.trim().is_empty() {
        return Err(
            ManagerError::EmptyReason
        );
    }

    let override_type = match approval {
        ApprovalStatus::Approved => {
            ApprovalOverride::ForceApproved
        }

        ApprovalStatus::Rejected => {
            ApprovalOverride::ForceRejected
        }

        ApprovalStatus::NoResponse => {
            return Err(
                ManagerError::OverrideNotPermitted
            );
        }
    };

    Ok(ApprovalDecision {
        booking_id: booking.id,
        manager_user_id: manager.user_id,
        approval_status: approval,
        rule_result: evaluation.result,
        override_type,
        reason,
    })
}

pub fn override_rules(
    manager: &ManagerAccount,
    booking: &Booking,
    new_result: RuleResult,
    reason: String,
) -> Result<ApprovalDecision, ManagerError> {
    if !can_perform_action(
        manager,
        booking,
        ManagerAction::OverrideRules,
    ) {
        return Err(
            ManagerError::OverrideNotPermitted
        );
    }

    if reason.trim().is_empty() {
        return Err(
            ManagerError::EmptyReason
        );
    }

    Ok(ApprovalDecision {
        booking_id: booking.id,
        manager_user_id: manager.user_id,
        approval_status: ApprovalStatus::NoResponse,
        rule_result: new_result,
        override_type: ApprovalOverride::RuleOverride,
        reason,
    })
}
