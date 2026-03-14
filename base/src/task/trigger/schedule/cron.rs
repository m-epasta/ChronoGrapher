use crate::errors::{
    CronError, CronErrorTypes, CronExpressionLexerErrors, CronExpressionParserErrors,
};
use crate::task::schedule::TaskSchedule;
use crate::task::schedule::cron_lexer::{Token, tokenize_fields};
use crate::task::schedule::cron_parser::{AstNode, AstTreeNode, CronParser};
use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc, Weekday};
use std::error::Error;
use std::ops::RangeInclusive;
use std::str::FromStr;
use std::time::SystemTime;

const RANGES: [RangeInclusive<u8>; 6] = [
    0..=59u8,
    0..=59u8,
    0..=23u8,
    1u8..=31u8,
    1u8..=12u8,
    1u8..=7u8,
];

#[derive(Clone, Eq, PartialEq, Default)]
pub enum CronField {
    #[default]
    Wildcard,

    Exact(u8),
    Range(u8, u8),
    Step(Box<CronField>, u8),
    List(Box<[CronField]>),
    Unspecified,
    Last(Option<i8>),
    NearestWeekday(u8),
    NthWeekday(u8, u8),
}

impl CronField {
    pub fn is_satisfied(&self, val: u8, range: &RangeInclusive<u8>) -> bool {
        match self {
            CronField::Wildcard | CronField::Unspecified => true,
            CronField::Exact(v) => *v == val,
            CronField::Range(start, end) => val >= *start && val <= *end,
            CronField::Step(base, step) => {
                if !base.is_satisfied(val, range) {
                    return false;
                }
                let start = match **base {
                    CronField::Range(s, _) => s,
                    CronField::Exact(v) => v,
                    _ => *range.start(),
                };
                (val >= start) && (val - start) % *step == 0
            }
            CronField::List(list) => list.iter().any(|f| f.is_satisfied(val, range)),
            _ => false, // Handled by structural matches in schedule
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct TaskScheduleCron {
    pub seconds: CronField,
    pub minute: CronField,
    pub hour: CronField,
    pub day_of_month: CronField,
    pub month: CronField,
    pub day_of_week: CronField,
}

impl TaskScheduleCron {
    pub fn new(cron: [CronField; 6]) -> Self {
        let [seconds, minute, hour, day_of_month, month, day_of_week] = cron;
        Self {
            seconds,
            minute,
            hour,
            day_of_month,
            month,
            day_of_week,
        }
    }

    fn map_ast_to_field(
        node: &AstNode,
        field_pos: usize,
        range: &RangeInclusive<u8>,
    ) -> Result<CronField, CronError> {
        let err = |error_type: CronErrorTypes| CronError {
            field_pos,
            position: node.start,
            error_type,
        };

        match &node.kind {
            AstTreeNode::Wildcard => Ok(CronField::Wildcard),
            AstTreeNode::Exact(v) => {
                if range.contains(v) {
                    Ok(CronField::Exact(*v))
                } else {
                    Err(err(CronErrorTypes::Lexer(
                        CronExpressionLexerErrors::InvalidNumericRange {
                            num: *v,
                            start: *range.start(),
                            end: *range.end(),
                        },
                    )))
                }
            }
            AstTreeNode::Range(start_node, end_node) => {
                let start = if let AstTreeNode::Exact(v) = start_node.kind {
                    v
                } else {
                    return Err(err(CronErrorTypes::Parser(
                        CronExpressionParserErrors::ExpectedNumber,
                    )));
                };
                let end = if let AstTreeNode::Exact(v) = end_node.kind {
                    v
                } else {
                    return Err(err(CronErrorTypes::Parser(
                        CronExpressionParserErrors::ExpectedNumber,
                    )));
                };
                if !range.contains(&start) {
                    return Err(err(CronErrorTypes::Lexer(
                        CronExpressionLexerErrors::InvalidNumericRange {
                            num: start,
                            start: *range.start(),
                            end: *range.end(),
                        },
                    )));
                }
                if !range.contains(&end) {
                    return Err(err(CronErrorTypes::Lexer(
                        CronExpressionLexerErrors::InvalidNumericRange {
                            num: end,
                            start: *range.start(),
                            end: *range.end(),
                        },
                    )));
                }
                if start > end {
                    return Err(err(CronErrorTypes::Lexer(
                        CronExpressionLexerErrors::InvalidRangeBounds { start, end },
                    )));
                }
                Ok(CronField::Range(start, end))
            }
            AstTreeNode::Step(base_node, step) => {
                let base = Self::map_ast_to_field(base_node, field_pos, range)?;
                Ok(CronField::Step(Box::new(base), *step))
            }
            AstTreeNode::List(nodes) => {
                let mut items = Vec::new();
                for n in nodes {
                    items.push(Self::map_ast_to_field(n, field_pos, range)?);
                }
                Ok(CronField::List(items.into_boxed_slice()))
            }
            AstTreeNode::LastOf(opt) => Ok(CronField::Last(opt.map(|v| v as i8))),
            AstTreeNode::NearestWeekday(expr_node) => {
                if let AstTreeNode::Exact(v) = expr_node.kind {
                    if !range.contains(&v) {
                        return Err(err(CronErrorTypes::Lexer(
                            CronExpressionLexerErrors::InvalidNumericRange {
                                num: v,
                                start: *range.start(),
                                end: *range.end(),
                            },
                        )));
                    }
                    Ok(CronField::NearestWeekday(v))
                } else {
                    Err(err(CronErrorTypes::Parser(
                        CronExpressionParserErrors::ExpectedNumber,
                    )))
                }
            }
            AstTreeNode::NthWeekday(day, nth) => Ok(CronField::NthWeekday(*day, *nth)),
            AstTreeNode::Unspecified => Ok(CronField::Unspecified),
        }
    }

    fn next_month(&self, dt: DateTime<Utc>) -> DateTime<Utc> {
        let year = if dt.month() == 12 {
            dt.year() + 1
        } else {
            dt.year()
        };
        let month = if dt.month() == 12 { 1 } else { dt.month() + 1 };
        Utc.with_ymd_and_hms(year, month, 1, 0, 0, 0).unwrap()
    }

    fn next_day(&self, dt: DateTime<Utc>) -> DateTime<Utc> {
        let tomorrow = dt
            .date_naive()
            .succ_opt()
            .unwrap_or_else(|| dt.date_naive());
        Utc.from_utc_datetime(&tomorrow.and_hms_opt(0, 0, 0).unwrap())
    }

    fn next_hour(&self, dt: DateTime<Utc>) -> DateTime<Utc> {
        let next = dt + chrono::Duration::hours(1);
        Utc.with_ymd_and_hms(next.year(), next.month(), next.day(), next.hour(), 0, 0)
            .unwrap()
    }

    fn next_minute(&self, dt: DateTime<Utc>) -> DateTime<Utc> {
        let next = dt + chrono::Duration::minutes(1);
        Utc.with_ymd_and_hms(
            next.year(),
            next.month(),
            next.day(),
            next.hour(),
            next.minute(),
            0,
        )
        .unwrap()
    }

    fn next_second(&self, dt: DateTime<Utc>) -> DateTime<Utc> {
        dt + chrono::Duration::seconds(1)
    }

    fn is_day_satisfied(&self, dt: DateTime<Utc>) -> bool {
        let dom_is_wild = matches!(
            self.day_of_month,
            CronField::Wildcard | CronField::Unspecified
        );
        let dow_is_wild = matches!(
            self.day_of_week,
            CronField::Wildcard | CronField::Unspecified
        );

        let dom_matches = self.day_of_month_matches(dt);
        let dow_matches = self.day_of_week_matches(dt);

        if dom_is_wild && dow_is_wild {
            true
        } else if dom_is_wild {
            dow_matches
        } else if dow_is_wild {
            dom_matches
        } else {
            dom_matches || dow_matches
        }
    }

    fn day_of_month_matches(&self, dt: DateTime<Utc>) -> bool {
        let val = dt.day() as u8;
        match &self.day_of_month {
            CronField::Last(None) => val == self.get_last_day_of_month(dt),
            CronField::NearestWeekday(d) => val == self.get_nearest_weekday(dt, *d),
            f => f.is_satisfied(val, &RANGES[3]),
        }
    }

    fn day_of_week_matches(&self, dt: DateTime<Utc>) -> bool {
        let val = (dt.weekday().num_days_from_sunday() + 1) as u8;
        match &self.day_of_week {
            CronField::Last(Some(d)) => val == *d as u8 && self.is_last_weekday_of_month(dt),
            CronField::NthWeekday(d, n) => val == *d && self.is_nth_weekday_of_month(dt, *n),
            f => f.is_satisfied(val, &RANGES[5]),
        }
    }

    fn get_last_day_of_month(&self, dt: DateTime<Utc>) -> u8 {
        let next_month = if dt.month() == 12 {
            Utc.with_ymd_and_hms(dt.year() + 1, 1, 1, 0, 0, 0).unwrap()
        } else {
            Utc.with_ymd_and_hms(dt.year(), dt.month() + 1, 1, 0, 0, 0)
                .unwrap()
        };
        next_month.naive_utc().date().pred_opt().unwrap().day() as u8
    }

    fn is_last_weekday_of_month(&self, dt: DateTime<Utc>) -> bool {
        dt.day() + 7 > self.get_last_day_of_month(dt) as u32
    }

    fn is_nth_weekday_of_month(&self, dt: DateTime<Utc>, n: u8) -> bool {
        (dt.day() - 1) / 7 + 1 == (n as u32)
    }

    fn get_nearest_weekday(&self, dt: DateTime<Utc>, target_day: u8) -> u8 {
        let date = Utc
            .with_ymd_and_hms(dt.year(), dt.month(), target_day as u32, 0, 0, 0)
            .unwrap();
        let weekday = date.weekday();
        if weekday != Weekday::Sat && weekday != Weekday::Sun {
            return target_day;
        }

        if weekday == Weekday::Sat {
            if target_day > 1 { target_day - 1 } else { 3 }
        } else {
            let last = self.get_last_day_of_month(dt);
            if target_day < last {
                target_day + 1
            } else {
                target_day - 2
            }
        }
    }
}

impl FromStr for TaskScheduleCron {
    type Err = CronError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = tokenize_fields(s).map_err(|(error_type, position, field_pos)| CronError {
            field_pos,
            position,
            error_type: CronErrorTypes::Lexer(error_type),
        })?;

        let mut ast: [AstNode; 6] = Default::default();
        let mut prev_toks: &[Token] = &tokens[0];
        for (idx, toks) in tokens.iter().enumerate() {
            if toks.len() == 0 {
                ast[idx] = AstNode {
                    start: if prev_toks.is_empty() {
                        0
                    } else {
                        prev_toks.last().unwrap().start
                    },
                    kind: AstTreeNode::Wildcard,
                };
                prev_toks = &toks;
                continue;
            }
            let mut parser_instance = CronParser::new(&toks);
            ast[idx] = parser_instance.parse_field().map_err(|error_type| {
                let pos = if parser_instance.pos < toks.len() {
                    toks[parser_instance.pos].start
                } else if !toks.is_empty() {
                    toks.last().unwrap().start
                } else {
                    0
                };
                CronError {
                    field_pos: idx,
                    position: pos,
                    error_type: CronErrorTypes::Parser(error_type),
                }
            })?;

            prev_toks = &toks;
        }

        let mut fields: [CronField; 6] = Default::default();
        for (idx, node) in ast.iter().enumerate() {
            fields[idx] = Self::map_ast_to_field(node, idx, &RANGES[idx])?;
        }

        Ok(Self::new(fields))
    }
}

impl TaskSchedule for TaskScheduleCron {
    fn schedule(&self, time: SystemTime) -> Result<SystemTime, Box<dyn Error + Send + Sync>> {
        let mut dt: DateTime<Utc> = time.into();
        dt = dt + chrono::Duration::seconds(1);

        let limit = dt + chrono::Duration::days(365 * 5);

        while dt < limit {
            if !self.month.is_satisfied(dt.month() as u8, &RANGES[4]) {
                dt = self.next_month(dt);
                continue;
            }

            if !self.is_day_satisfied(dt) {
                dt = self.next_day(dt);
                continue;
            }

            if !self.hour.is_satisfied(dt.hour() as u8, &RANGES[2]) {
                dt = self.next_hour(dt);
                continue;
            }

            if !self.minute.is_satisfied(dt.minute() as u8, &RANGES[1]) {
                dt = self.next_minute(dt);
                continue;
            }

            if !self.seconds.is_satisfied(dt.second() as u8, &RANGES[0]) {
                dt = self.next_second(dt);
                continue;
            }

            return Ok(dt.into());
        }

        Err("No matching time found within 5 years".into())
    }
}
