use chrono::NaiveDate;

use crate::calendar::is_rochdale_term_time;

#[derive(Debug, PartialEq, Clone)]
pub enum BookingType {
    Holiday,
    Sick,
    ExceptionalCircumstance,
    WorkLeave,
    Appointment,
    WorkFromHome,
    CharityBuilding,
    StaffEvent,
    Other(String),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PaymentStatus {
    NoResponse,
    Paid,
    Unpaid,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ApprovalStatus {
    NoResponse,
    Approved,
    Rejected,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RuleResult {
    Allowed,
    RuleBreaker,
    ExceptionalRuleBreaker,
}

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub enum Department {
    Trading,
    Accounting,
    IT,
    Logistics,
    Directors,
    EBFT,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EbftRole {
    Jac,
    Steve,
    Travis,
    Other,
}

#[derive(Debug, Clone)]
pub struct EmployeeInfo {
    pub id: u32,
    pub name: String,
    pub holiday_allowance: u32,
    pub department: Department,
    pub additional_departments: Vec<Department>,
    pub ebft_role: Option<EbftRole>,
    pub manager_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Booking {
    pub id: u32,
    pub employee_id: u32,
    pub date: NaiveDate,
    pub booking_type: BookingType,
    pub department_id: u32,
    pub payment_status: PaymentStatus,
    pub approval_status: ApprovalStatus,
}

#[derive(Debug, Clone)]
pub struct DepartmentRule {
    pub department: Department,
    pub normal_absence_limit: u32,
    pub exceptional_can_exceed_limit: bool,
}

#[derive(Debug, Clone)]
pub struct BookingContext {
    pub employee: EmployeeInfo,
    pub department_rule: DepartmentRule,
    pub secondary_department_rule: Option<DepartmentRule>,
}

#[derive(Debug)]
pub struct BookingEvaluation {
    pub result: RuleResult,
    pub reasons: Vec<String>,
    pub approver_id: Option<u32>,
    pub requires_approval: bool,
    pub is_exceptional: bool,
    pub holiday_days_remaining: Option<u32>,
    pub requires_head_trustee_approval: bool,
}

pub fn counts_towards_department_limit(
    booking_type: &BookingType,
) -> bool {
    match booking_type {
        BookingType::Holiday => true,
        BookingType::WorkLeave => true,
        BookingType::ExceptionalCircumstance => false,
        BookingType::Sick => false,
        BookingType::Appointment => false,
        BookingType::WorkFromHome => false,
        BookingType::CharityBuilding => false,
        BookingType::StaffEvent => false,
        BookingType::Other(_) => false,
    }
}

pub fn count_department_bookings(
    bookings: &[Booking],
    department: Department,
    date: NaiveDate,
    employees: &[EmployeeInfo],
) -> u32 {
    bookings
        .iter()
        .filter(|booking| booking.date == date)
        .filter(|booking| {
            let employee = employees
                .iter()
                .find(|employee| employee.id == booking.employee_id);

            match employee {
                Some(employee) => {
                    employee.department == department
                        || employee
                            .additional_departments
                            .contains(&department)
                }
                None => false,
            }
        })
        .filter(|booking| {
            counts_towards_department_limit(
                &booking.booking_type,
            )
        })
        .count() as u32
}

pub fn meets_notice_requirement(
    days_until_request: i64,
    booking_type: &BookingType,
) -> bool {
    if *booking_type == BookingType::ExceptionalCircumstance {
        return true;
    }

    days_until_request >= 14
}

pub fn holidays_used(
    bookings: &[Booking],
    employee_id: u32,
) -> u32 {
    bookings
        .iter()
        .filter(|booking| booking.employee_id == employee_id)
        .filter(|booking| {
            booking.booking_type == BookingType::Holiday
        })
        .count() as u32
}

pub fn holidays_remaining(
    bookings: &[Booking],
    employee: &EmployeeInfo,
) -> u32 {
    employee
        .holiday_allowance
        .saturating_sub(
            holidays_used(
                bookings,
                employee.id,
            )
        )
}

fn employee_by_id<'a>(
    employees: &'a [EmployeeInfo],
    employee_id: u32,
) -> Option<&'a EmployeeInfo> {
    employees
        .iter()
        .find(|employee| employee.id == employee_id)
}

fn employee_has_department(
    employee: &EmployeeInfo,
    department: Department,
) -> bool {
    employee.department == department
        || employee
            .additional_departments
            .contains(&department)
}

fn count_department_absences(
    bookings: &[Booking],
    employees: &[EmployeeInfo],
    department: Department,
    date: NaiveDate,
) -> u32 {
    count_department_bookings(
        bookings,
        department,
        date,
        employees,
    )
}

fn check_department_rule(
    bookings: &[Booking],
    employees: &[EmployeeInfo],
    new_booking: &Booking,
    rule: &DepartmentRule,
) -> Option<(RuleResult, String)> {
    let current = count_department_absences(
        bookings,
        employees,
        rule.department,
        new_booking.date,
    );

    if current < rule.normal_absence_limit {
        return Some((
            RuleResult::Allowed,
            format!(
                "The department has {} of {} normal absence positions occupied.",
                current,
                rule.normal_absence_limit
            ),
        ));
    }

    if new_booking.booking_type
        == BookingType::ExceptionalCircumstance
        && rule.exceptional_can_exceed_limit
    {
        return Some((
            RuleResult::ExceptionalRuleBreaker,
            "The normal absence limit has been reached. This exceptional circumstance request is a yellow exception."
                .to_string(),
        ));
    }

    Some((
        RuleResult::RuleBreaker,
        "The normal absence limit for this department has been reached."
            .to_string(),
    ))
}

fn count_director_absences(
    bookings: &[Booking],
    employees: &[EmployeeInfo],
    date: NaiveDate,
) -> u32 {
    bookings
        .iter()
        .filter(|booking| booking.date == date)
        .filter(|booking| {
            employee_by_id(
                employees,
                booking.employee_id,
            )
            .map(|employee| {
                employee_has_department(
                    employee,
                    Department::Directors,
                )
            })
            .unwrap_or(false)
        })
        .filter(|booking| {
            counts_towards_department_limit(
                &booking.booking_type,
            )
        })
        .count() as u32
}

fn check_director_rule(
    bookings: &[Booking],
    employees: &[EmployeeInfo],
    new_booking: &Booking,
    rule: &DepartmentRule,
) -> Option<(RuleResult, String)> {
    let employee = employee_by_id(
        employees,
        new_booking.employee_id,
    )?;

    if !employee_has_department(
        employee,
        Department::Directors,
    ) {
        return None;
    }

    let current = count_director_absences(
        bookings,
        employees,
        new_booking.date,
    );

    if current < rule.normal_absence_limit {
        return Some((
            RuleResult::Allowed,
            format!(
                "Directors has {} of {} absence positions occupied.",
                current,
                rule.normal_absence_limit
            ),
        ));
    }

    if new_booking.booking_type
        == BookingType::ExceptionalCircumstance
        && rule.exceptional_can_exceed_limit
    {
        return Some((
            RuleResult::ExceptionalRuleBreaker,
            "The Directors' absence limit has been reached. This exceptional circumstance request is a yellow exception."
                .to_string(),
        ));
    }

    Some((
        RuleResult::RuleBreaker,
        "The Directors' absence limit has been reached."
            .to_string(),
    ))
}

fn check_ebft_rule(
    bookings: &[Booking],
    employees: &[EmployeeInfo],
    new_booking: &Booking,
    rule: &DepartmentRule,
) -> Option<(RuleResult, String, bool)> {
    if rule.department != Department::EBFT {
        return None;
    }

    let employee = employee_by_id(
        employees,
        new_booking.employee_id,
    )?;

    let term_time = is_rochdale_term_time(
        new_booking.date,
    );

    if term_time {
        let current = count_department_absences(
            bookings,
            employees,
            Department::EBFT,
            new_booking.date,
        );

        if current < rule.normal_absence_limit {
            return Some((
                RuleResult::Allowed,
                "EBFT is in Rochdale term time and has an absence position available."
                    .to_string(),
                false,
            ));
        }

        if new_booking.booking_type
            == BookingType::ExceptionalCircumstance
            && rule.exceptional_can_exceed_limit
        {
            return Some((
                RuleResult::ExceptionalRuleBreaker,
                "EBFT has reached its normal absence limit during Rochdale term time. This exceptional circumstance request is a yellow exception."
                    .to_string(),
                false,
            ));
        }

        return Some((
            RuleResult::RuleBreaker,
            "EBFT has reached its normal absence limit during Rochdale term time."
                .to_string(),
            false,
        ));
    }

    match employee.ebft_role {
        Some(EbftRole::Jac) => {
            Some((
                RuleResult::Allowed,
                "The employee may be absent outside Rochdale term time."
                    .to_string(),
                false,
            ))
        }

        Some(EbftRole::Steve) |
        Some(EbftRole::Travis) => {
            let jac_off = bookings
                .iter()
                .filter(|booking| {
                    booking.date == new_booking.date
                })
                .filter_map(|booking| {
                    employee_by_id(
                        employees,
                        booking.employee_id,
                    )
                })
                .any(|employee| {
                    employee.department == Department::EBFT
                        && employee.ebft_role
                            == Some(EbftRole::Jac)
                        && bookings.iter().any(|booking| {
                            booking.employee_id == employee.id
                                && booking.date == new_booking.date
                                && counts_towards_department_limit(
                                    &booking.booking_type,
                                )
                        })
                });

            if jac_off {
                return Some((
                    RuleResult::Allowed,
                    "The employee may be absent because Jac is also absent."
                        .to_string(),
                    false,
                ));
            }

            let other_role =
                match employee.ebft_role {
                    Some(EbftRole::Steve) => EbftRole::Travis,
                    Some(EbftRole::Travis) => EbftRole::Steve,
                    _ => return None,
                };

            let other_is_off = bookings
                .iter()
                .filter(|booking| {
                    booking.date == new_booking.date
                })
                .filter_map(|booking| {
                    employee_by_id(
                        employees,
                        booking.employee_id,
                    )
                })
                .any(|employee| {
                    employee.department == Department::EBFT
                        && employee.ebft_role
                            == Some(other_role)
                        && bookings.iter().any(|booking| {
                            booking.employee_id == employee.id
                                && booking.date == new_booking.date
                                && counts_towards_department_limit(
                                    &booking.booking_type,
                                )
                        })
                });

            if other_is_off {
                return Some((
                    RuleResult::Allowed,
                    "Steve and Travis are both absent outside term time. Head Trustee approval is required."
                        .to_string(),
                    true,
                ));
            }

            Some((
                RuleResult::Allowed,
                "The employee may be absent individually outside Rochdale term time."
                    .to_string(),
                false,
            ))
        }

        Some(EbftRole::Other) |
        None => {
            Some((
                RuleResult::Allowed,
                "The EBFT request is allowed outside Rochdale term time."
                    .to_string(),
                false,
            ))
        }
    }
}

fn check_holiday_allowance(
    bookings: &[Booking],
    employee: &EmployeeInfo,
    new_booking: &Booking,
) -> Option<(RuleResult, String)> {
    if new_booking.booking_type
        != BookingType::Holiday
    {
        return None;
    }

    let used = holidays_used(
        bookings,
        employee.id,
    );

    if used < employee.holiday_allowance {
        let remaining =
            employee.holiday_allowance - used;

        return Some((
            RuleResult::Allowed,
            format!(
                "Employee has {} holiday days remaining before this request.",
                remaining
            ),
        ));
    }

    Some((
        RuleResult::RuleBreaker,
        "The employee has no holiday allowance remaining."
            .to_string(),
    ))
}

fn merge_rule_result(
    current: &mut RuleResult,
    incoming: RuleResult,
) {
    match incoming {
        RuleResult::RuleBreaker => {
            *current = RuleResult::RuleBreaker;
        }

        RuleResult::ExceptionalRuleBreaker => {
            if *current != RuleResult::RuleBreaker {
                *current =
                    RuleResult::ExceptionalRuleBreaker;
            }
        }

        RuleResult::Allowed => {}
    }
}

pub fn evaluate_booking(
    bookings: &[Booking],
    employees: &[EmployeeInfo],
    context: &BookingContext,
    new_booking: &Booking,
    days_until_request: i64,
) -> BookingEvaluation {
    let employee = &context.employee;

    let mut reasons =
        Vec::<String>::new();

    let mut final_result =
        RuleResult::Allowed;

    let mut requires_head_trustee_approval =
        false;

    if !meets_notice_requirement(
        days_until_request,
        &new_booking.booking_type,
    ) {
        reasons.push(
            "This request was made with less than 14 days notice."
                .to_string(),
        );

        final_result =
            RuleResult::RuleBreaker;
    }

    if let Some((result, reason)) =
        check_holiday_allowance(
            bookings,
            employee,
            new_booking,
        )
    {
        reasons.push(reason);

        merge_rule_result(
            &mut final_result,
            result,
        );
    }

    if context.department_rule.department
        == Department::EBFT
    {
        if let Some((
            result,
            reason,
            trustee_required,
        )) = check_ebft_rule(
            bookings,
            employees,
            new_booking,
            &context.department_rule,
        )
        {
            reasons.push(reason);

            if trustee_required {
                requires_head_trustee_approval =
                    true;
            }

            merge_rule_result(
                &mut final_result,
                result,
            );
        }
    } else {
        let employee_belongs_to_department =
            employee_has_department(
                employee,
                context.department_rule.department,
            );

        if employee_belongs_to_department {
            if let Some((
                result,
                reason,
            )) = check_department_rule(
                bookings,
                employees,
                new_booking,
                &context.department_rule,
            )
            {
                reasons.push(reason);

                merge_rule_result(
                    &mut final_result,
                    result,
                );
            }
        }
    }

    if let Some(secondary_rule) =
        &context.secondary_department_rule
    {
        if employee_has_department(
            employee,
            secondary_rule.department,
        ) {
            if secondary_rule.department
                == Department::Directors
            {
                if let Some((
                    result,
                    reason,
                )) = check_director_rule(
                    bookings,
                    employees,
                    new_booking,
                    secondary_rule,
                )
                {
                    reasons.push(reason);

                    merge_rule_result(
                        &mut final_result,
                        result,
                    );
                }
            } else {
                if let Some((
                    result,
                    reason,
                )) = check_department_rule(
                    bookings,
                    employees,
                    new_booking,
                    secondary_rule,
                )
                {
                    reasons.push(reason);

                    merge_rule_result(
                        &mut final_result,
                        result,
                    );
                }
            }
        }
    }

    let holiday_days_remaining =
        if new_booking.booking_type
            == BookingType::Holiday
        {
            Some(
                employee
                    .holiday_allowance
                    .saturating_sub(
                        holidays_used(
                            bookings,
                            employee.id,
                        ) + 1,
                    ),
            )
        } else {
            None
        };

    BookingEvaluation {
        result: final_result,
        reasons,
        approver_id: employee.manager_id,
        requires_approval: true,
        is_exceptional:
            new_booking.booking_type
                == BookingType::ExceptionalCircumstance,
        holiday_days_remaining,
        requires_head_trustee_approval,
    }
}

pub fn evaluate_booking_result(
    bookings: &[Booking],
    employees: &[EmployeeInfo],
    context: &BookingContext,
    new_booking: &Booking,
    days_until_request: i64,
) -> RuleResult {
    evaluate_booking(
        bookings,
        employees,
        context,
        new_booking,
        days_until_request,
    )
    .result
}

