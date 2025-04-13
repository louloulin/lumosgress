use crate::services::compliance::{ComplianceReport, ReportType, ReportStatus, ViolationSeverity};
use chrono::{DateTime, Utc};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ComplianceReportProps {
    pub report: ComplianceReport,
    pub on_status_change: Callback<(String, ReportStatus)>,
}

#[function_component(ComplianceReportComponent)]
pub fn compliance_report(props: &ComplianceReportProps) -> Html {
    let report = &props.report;
    let on_status_change = props.on_status_change.clone();

    let status_change = {
        let report_id = report.id.clone();
        Callback::from(move |status: ReportStatus| {
            on_status_change.emit((report_id.clone(), status));
        })
    };

    html! {
        <div class="compliance-report">
            <div class="report-header">
                <h2>{ format!("Compliance Report: {}", report.report_type) }</h2>
                <div class="report-meta">
                    <span>{ format!("Generated: {}", report.generated_at) }</span>
                    <span>{ format!("Period: {} to {}", report.start_time, report.end_time) }</span>
                </div>
            </div>

            <div class="report-status">
                <select
                    value={report.status.to_string()}
                    onchange={move |e: Event| {
                        let select = e.target_unchecked_into::<web_sys::HtmlSelectElement>();
                        if let Ok(status) = select.value().parse::<ReportStatus>() {
                            status_change.emit(status);
                        }
                    }}
                >
                    <option value="Pending">{ "Pending" }</option>
                    <option value="Reviewed">{ "Reviewed" }</option>
                    <option value="Resolved">{ "Resolved" }</option>
                </select>
            </div>

            <div class="violations">
                <h3>{ "Violations" }</h3>
                { for report.violations.iter().map(|violation| {
                    let severity_class = match violation.severity {
                        ViolationSeverity::Critical => "critical",
                        ViolationSeverity::High => "high",
                        ViolationSeverity::Medium => "medium",
                        ViolationSeverity::Low => "low",
                    };
                    html! {
                        <div class={classes!("violation", severity_class)}>
                            <div class="violation-header">
                                <span class="severity">{ format!("{:?}", violation.severity) }</span>
                                <span class="timestamp">{ violation.timestamp.to_string() }</span>
                            </div>
                            <div class="violation-description">
                                { &violation.description }
                            </div>
                            <div class="violation-details">
                                { &violation.details }
                            </div>
                        </div>
                    }
                })}
            </div>

            <div class="report-summary">
                <h3>{ "Summary" }</h3>
                <div class="summary-stats">
                    <div class="stat">
                        <span class="label">{ "Total Violations:" }</span>
                        <span class="value">{ report.violations.len() }</span>
                    </div>
                    <div class="stat">
                        <span class="label">{ "Critical:" }</span>
                        <span class="value">{ report.violations.iter().filter(|v| v.severity == ViolationSeverity::Critical).count() }</span>
                    </div>
                    <div class="stat">
                        <span class="label">{ "High:" }</span>
                        <span class="value">{ report.violations.iter().filter(|v| v.severity == ViolationSeverity::High).count() }</span>
                    </div>
                    <div class="stat">
                        <span class="label">{ "Medium:" }</span>
                        <span class="value">{ report.violations.iter().filter(|v| v.severity == ViolationSeverity::Medium).count() }</span>
                    </div>
                    <div class="stat">
                        <span class="label">{ "Low:" }</span>
                        <span class="value">{ report.violations.iter().filter(|v| v.severity == ViolationSeverity::Low).count() }</span>
                    </div>
                </div>
            </div>
        </div>
    }
} 