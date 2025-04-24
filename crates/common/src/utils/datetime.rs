//! 时间工具模块
//! 提供常用的时间操作、计算和格式化功能
use chrono::offset::LocalResult;
use chrono::{
    DateTime, Datelike, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike,
    Utc, Weekday,
};

/// 常用日期时间格式常量
pub mod formats {
    /// 标准日期时间格式: YYYY-MM-DD HH:MM:SS
    pub const DATETIME: &str = "%Y-%m-%d %H:%M:%S";
    /// 仅日期格式: YYYY-MM-DD
    pub const DATE: &str = "%Y-%m-%d";
    /// 仅时间格式: HH:MM:SS
    pub const TIME: &str = "%H:%M:%S";
    /// ISO 日期时间格式: YYYY-MM-DDThh:mm:ss
    pub const ISO_DATETIME: &str = "%Y-%m-%dT%H:%M:%S";
    /// 紧凑日期时间格式: YYYYMMDDhhmmss
    pub const DATETIME_COMPACT: &str = "%Y%m%d%H%M%S";
    /// 紧凑日期格式: YYYYMMDD
    pub const DATE_COMPACT: &str = "%Y%m%d";

    /// 反序列化支持的格式列表
    pub const ACCEPTED_FORMATS: &[&str] = &[
        DATETIME,            // 标准格式
        ISO_DATETIME,        // ISO格式
        "%Y-%m-%d %H:%M",    // 没有秒
        "%Y/%m/%d %H:%M:%S", // 斜杠分隔的日期
    ];
}

/// 时间工具类
pub struct TimeUtil;

impl TimeUtil {
    /// 获取当前本地时间
    pub fn now() -> DateTime<Local> {
        Local::now()
    }

    /// 获取当前UTC时间
    pub fn now_utc() -> DateTime<Utc> {
        Utc::now()
    }

    /// 获取当前时间戳（秒）
    pub fn timestamp() -> i64 {
        Self::now().timestamp()
    }

    /// 获取当前时间戳（毫秒）
    pub fn timestamp_millis() -> i64 {
        Self::now().timestamp_millis()
    }

    /// 获取当前朴素日期时间 (NaiveDateTime)
    pub fn now_naive() -> NaiveDateTime {
        Self::now().naive_local()
    }

    /// 格式化日期时间为字符串
    pub fn format(dt: DateTime<Local>, fmt: &str) -> String {
        dt.format(fmt).to_string()
    }

    /// 获取当前日期时间字符串，使用标准格式：YYYY-MM-DD HH:MM:SS
    pub fn now_string() -> String {
        Self::format(Self::now(), formats::DATETIME)
    }

    /// 获取当前日期字符串，格式：YYYY-MM-DD
    pub fn today_string() -> String {
        Self::format(Self::now(), formats::DATE)
    }

    /// 获取当前时间字符串，格式：HH:MM:SS
    pub fn time_string() -> String {
        Self::format(Self::now(), formats::TIME)
    }

    /// 格式化时间戳为标准格式字符串
    pub fn format_timestamp(timestamp: i64) -> String {
        match Local.timestamp_opt(timestamp, 0) {
            LocalResult::Single(dt) => Self::format(dt, formats::DATETIME),
            _ => "Invalid timestamp".to_string(),
        }
    }

    /// 获取特定小时前/后的时间
    pub fn hours_from_now(hours: i64) -> DateTime<Local> {
        Self::now() + Duration::hours(hours)
    }

    /// 获取1小时前的时间
    pub fn one_hour_ago() -> DateTime<Local> {
        Self::hours_from_now(-1)
    }

    /// 获取特定天数前/后的时间
    pub fn days_from_now(days: i64) -> DateTime<Local> {
        Self::now() + Duration::days(days)
    }

    /// 获取昨天的时间
    pub fn yesterday() -> DateTime<Local> {
        Self::days_from_now(-1)
    }

    /// 获取明天的时间
    pub fn tomorrow() -> DateTime<Local> {
        Self::days_from_now(1)
    }

    /// 获取特定周数前/后的时间
    pub fn weeks_from_now(weeks: i64) -> DateTime<Local> {
        Self::now() + Duration::weeks(weeks)
    }

    /// 获取特定月数前/后的时间
    pub fn months_from_now(months: i32) -> DateTime<Local> {
        let now = Self::now();
        let naive_month = add_months_to_date(now.naive_local(), months);
        match Local.from_local_datetime(&naive_month) {
            LocalResult::Single(dt) => dt,
            _ => now,
        }
    }

    /// 获取上个月的同一天
    pub fn last_month() -> DateTime<Local> {
        Self::months_from_now(-1)
    }

    /// 获取下个月的同一天
    pub fn next_month() -> DateTime<Local> {
        Self::months_from_now(1)
    }

    /// 获取3个月前的时间
    pub fn three_months_ago() -> DateTime<Local> {
        Self::months_from_now(-3)
    }

    /// 获取特定年数前/后的时间
    pub fn years_from_now(years: i32) -> DateTime<Local> {
        let now = Self::now();
        let year = now.year() + years;
        match Local.with_ymd_and_hms(
            year,
            now.month(),
            now.day(),
            now.hour(),
            now.minute(),
            now.second(),
        ) {
            LocalResult::Single(dt) => dt,
            _ => now,
        }
    }

    /// 获取去年的今天
    pub fn last_year() -> DateTime<Local> {
        Self::years_from_now(-1)
    }

    /// 获取明年的今天
    pub fn next_year() -> DateTime<Local> {
        Self::years_from_now(1)
    }

    /// 获取本周一的日期时间
    pub fn this_monday() -> DateTime<Local> {
        let now = Self::now();
        let weekday = now.weekday().num_days_from_monday();
        now - Duration::days(weekday as i64)
    }

    /// 获取本周末(周日)的日期时间
    pub fn this_sunday() -> DateTime<Local> {
        let monday = Self::this_monday();
        monday + Duration::days(6)
    }

    /// 获取特定工作日的日期时间
    pub fn day_of_week(weekday: Weekday) -> DateTime<Local> {
        let monday = Self::this_monday();
        monday + Duration::days(weekday.num_days_from_monday() as i64)
    }

    /// 获取本月第一天
    pub fn first_day_of_month() -> DateTime<Local> {
        let now = Self::now();
        match Local.with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0) {
            LocalResult::Single(dt) => dt,
            _ => now,
        }
    }

    /// 获取本月最后一天
    pub fn last_day_of_month() -> DateTime<Local> {
        let first_day = Self::first_day_of_month();
        let next_month = Self::months_from_now(1);
        let first_day_next_month =
            match Local.with_ymd_and_hms(next_month.year(), next_month.month(), 1, 0, 0, 0) {
                LocalResult::Single(dt) => dt,
                _ => return first_day,
            };
        first_day_next_month - Duration::days(1)
    }

    /// 获取上个月第一天
    pub fn first_day_of_last_month() -> DateTime<Local> {
        let last_month = Self::last_month();
        match Local.with_ymd_and_hms(last_month.year(), last_month.month(), 1, 0, 0, 0) {
            LocalResult::Single(dt) => dt,
            _ => Self::first_day_of_month(),
        }
    }

    /// 获取上个月最后一天
    pub fn last_day_of_last_month() -> DateTime<Local> {
        Self::first_day_of_month() - Duration::days(1)
    }

    /// 获取本季度第一天
    pub fn first_day_of_quarter() -> DateTime<Local> {
        let now = Self::now();
        let month = now.month();
        let quarter_month = ((month - 1) / 3) * 3 + 1;

        match Local.with_ymd_and_hms(now.year(), quarter_month, 1, 0, 0, 0) {
            LocalResult::Single(dt) => dt,
            _ => now,
        }
    }

    /// 获取本年度第一天
    pub fn first_day_of_year() -> DateTime<Local> {
        let now = Self::now();
        match Local.with_ymd_and_hms(now.year(), 1, 1, 0, 0, 0) {
            LocalResult::Single(dt) => dt,
            _ => now,
        }
    }

    /// 将时间戳转为本地时间
    pub fn from_timestamp(timestamp: i64) -> DateTime<Local> {
        match Local.timestamp_opt(timestamp, 0) {
            LocalResult::Single(dt) => dt,
            _ => Self::now(),
        }
    }

    /// 将字符串解析为本地时间
    pub fn parse_datetime(datetime_str: &str) -> Result<DateTime<Local>, chrono::ParseError> {
        // 尝试以多种格式解析
        for format in formats::ACCEPTED_FORMATS {
            if let Ok(naive) = NaiveDateTime::parse_from_str(datetime_str, format) {
                return Ok(Local.from_local_datetime(&naive).unwrap());
            }
        }

        // 如果只有日期部分，添加默认时间 00:00:00
        if let Ok(date) = NaiveDate::parse_from_str(datetime_str, formats::DATE) {
            let naive = date.and_hms_opt(0, 0, 0).unwrap();
            return Ok(Local.from_local_datetime(&naive).unwrap());
        }

        // 最后尝试标准格式
        let naive = NaiveDateTime::parse_from_str(datetime_str, formats::DATETIME)?;
        Ok(Local.from_local_datetime(&naive).unwrap())
    }

    /// 获取只有日期部分的DateTime (时间设为00:00:00)
    pub fn date_only(dt: DateTime<Local>) -> DateTime<Local> {
        match Local.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0) {
            LocalResult::Single(result) => result,
            _ => dt,
        }
    }

    /// 判断两个时间是否是同一天
    pub fn is_same_day(dt1: DateTime<Local>, dt2: DateTime<Local>) -> bool {
        dt1.year() == dt2.year() && dt1.month() == dt2.month() && dt1.day() == dt2.day()
    }

    /// 获取两个时间之间的天数差
    pub fn days_between(dt1: DateTime<Local>, dt2: DateTime<Local>) -> i64 {
        let date1 = Self::date_only(dt1);
        let date2 = Self::date_only(dt2);
        (date2 - date1).num_days()
    }

    /// 计算两个日期时间之间的差异（返回人类可读格式）
    pub fn human_readable_duration(dt1: DateTime<Local>, dt2: DateTime<Local>) -> String {
        let duration = if dt1 > dt2 { dt1 - dt2 } else { dt2 - dt1 };

        let days = duration.num_days();
        if days > 0 {
            return format!("{}天", days);
        }

        let hours = duration.num_hours();
        if hours > 0 {
            return format!("{}小时", hours);
        }

        let minutes = duration.num_minutes();
        if minutes > 0 {
            return format!("{}分钟", minutes);
        }

        format!("{}秒", duration.num_seconds())
    }

    /// 获取当天开始时间 (00:00:00)
    pub fn start_of_day() -> DateTime<Local> {
        let now = Self::now();
        let midnight = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        match now
            .date_naive()
            .and_time(midnight)
            .and_local_timezone(Local)
        {
            LocalResult::Single(dt) => dt,
            _ => now,
        }
    }

    /// 获取当天结束时间 (23:59:59)
    pub fn end_of_day() -> DateTime<Local> {
        let now = Self::now();
        let end_time = NaiveTime::from_hms_opt(23, 59, 59).unwrap();
        match now
            .date_naive()
            .and_time(end_time)
            .and_local_timezone(Local)
        {
            LocalResult::Single(dt) => dt,
            _ => now,
        }
    }
}

/// 内部辅助函数：向日期添加月份
fn add_months_to_date(date: NaiveDateTime, months: i32) -> NaiveDateTime {
    let mut year = date.year();
    let mut month = date.month() as i32 + months;

    // 处理月份溢出
    while month > 12 {
        month -= 12;
        year += 1;
    }

    while month < 1 {
        month += 12;
        year -= 1;
    }

    let month = month as u32;

    // 处理月末日期 (例如 1月31日 + 1个月 应该是 2月28/29日)
    let max_day = get_days_in_month(year, month);
    let day = std::cmp::min(date.day(), max_day);

    // 构建新日期时间
    let new_date = chrono::NaiveDate::from_ymd_opt(year, month, day).unwrap_or_else(|| date.date());

    new_date
        .and_hms_opt(date.hour(), date.minute(), date.second())
        .unwrap_or(date)
}

/// 获取指定年月的天数
fn get_days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            // 闰年2月29天，平年28天
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 30, // 默认30天，实际不应该到达这里
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use formats::DATETIME;

    #[test]
    fn time_util_example_usage() {
        // 获取当前时间
        let now = TimeUtil::now();
        println!("当前时间: {}", TimeUtil::now_string());

        // 相对时间
        println!(
            "1小时前: {}",
            TimeUtil::one_hour_ago().format("%Y-%m-%d %H:%M:%S")
        );
        println!(
            "3个月前: {}",
            TimeUtil::months_from_now(-3).format("%Y-%m-%d %H:%M:%S")
        );
        println!("明天: {}", TimeUtil::tomorrow().format("%Y-%m-%d"));

        // 特定时间点
        println!("本周一: {}", TimeUtil::this_monday().format("%Y-%m-%d"));
        println!(
            "本周五: {}",
            TimeUtil::day_of_week(Weekday::Fri).format("%Y-%m-%d")
        );
        println!(
            "本月第一天: {}",
            TimeUtil::first_day_of_month().format("%Y-%m-%d")
        );
        println!(
            "本月最后一天: {}",
            TimeUtil::last_day_of_month().format("%Y-%m-%d")
        );
        println!(
            "上个月第一天: {}",
            TimeUtil::first_day_of_last_month().format("%Y-%m-%d")
        );

        // 时间计算
        let last_week = TimeUtil::weeks_from_now(-1);
        let days = TimeUtil::days_between(last_week, now);
        println!("过去一周的天数: {}", days);

        // 人类可读时间差
        let duration = TimeUtil::human_readable_duration(last_week, now);
        println!("时间差: {}", duration);

        // 特定格式化
        println!("紧凑日期格式: {}", TimeUtil::format(now, "%Y%m%d"));

        // 日期解析
        let parsed = TimeUtil::parse_datetime("2023-04-10 15:30:00").unwrap();
        println!("解析的时间: {}", parsed);

        // 当天的起始和结束时间
        println!(
            "今天开始: {}",
            TimeUtil::start_of_day().format(DATETIME)
        );
        println!(
            "今天结束: {}",
            TimeUtil::end_of_day().format(DATETIME)
        );

        print!("now_naive: {}", TimeUtil::now_naive().to_string());

        // 使用场景
        // 日志记录和报表生成:
        let log_time = TimeUtil::now_string();
        println!("[{}] 操作执行成功", log_time);

        // 数据过滤
        // 获取最近7天的数据
        // let seven_days_ago = TimeUtil::days_from_now(-7);
        // let recent_orders = db.query("SELECT * FROM orders WHERE created_at > ?", seven_days_ago);

        // 获取上个月的报表数据
        let start = TimeUtil::first_day_of_last_month();
        let end = TimeUtil::last_day_of_last_month();
        // let report_data = generate_report(start, end);

        // 判断是否需要执行月初任务
        let today = TimeUtil::now();
        if TimeUtil::is_same_day(today, TimeUtil::first_day_of_month()) {
            // perform_monthly_tasks();
            println!("执行月初任务")
        }
    }
}
