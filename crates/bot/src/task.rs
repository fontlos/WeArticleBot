//! 定时任务(时刻表驱动)

use log::{info, warn};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio_util::sync::CancellationToken;

use crate::handler;

/// 默认时刻表(每日, 北京时间)
const DEFAULT_SCHEDULE: &str = "08:00,20:00";

/// 启动定时任务, 返回其 JoinHandle, 供 main 在停机时等待退出
pub fn spawn(shutdown: CancellationToken) -> tokio::task::JoinHandle<()> {
    tokio::spawn(run(shutdown))
}

/// 定时任务主循环: 等到下一个时刻点才执行一轮 sync + summary
async fn run(shutdown: CancellationToken) {
    // 定时任务产生的提示消息发往的会话(群聊或单聊), 需要在 .env 中配置
    let Ok(chat_id) = std::env::var("TASK_CHAT_ID") else {
        warn!("未配置 TASK_CHAT_ID, 定时任务不启动; 请在 .env 填入机器人所在会话 id");
        return;
    };

    let spec = std::env::var("TASK_SCHEDULE")
        .unwrap_or_else(|_| DEFAULT_SCHEDULE.to_string());
    let Some(schedule) = parse_schedule(&spec) else {
        warn!("TASK_SCHEDULE 格式无效: {spec:?}, 期望如 \"08:00,20:00\"; 定时任务不启动");
        return;
    };
    info!("定时任务已启动: 每日 {} 执行, 目标会话 {chat_id}", fmt_schedule(&schedule));

    loop {
        let now = unix_now();
        let next = next_run_at(&schedule, now);
        let wait = next.saturating_sub(now);
        info!("下次执行: {} ({} 后)", fmt_beijing_time(next), fmt_duration(wait));

        tokio::select! {
            _ = shutdown.cancelled() => {
                info!("定时任务已停止");
                return;
            }
            _ = tokio::time::sleep(Duration::from_secs(wait)) => {}
        }
        // 若等待期间收到停机信号, 直接退出(select 随机选择时兜底)
        if shutdown.is_cancelled() {
            info!("定时任务已停止");
            return;
        }

        run_pass(&chat_id).await;
    }
}

/// 执行一轮: 同步文章 -> 逐篇总结待总结文章
async fn run_pass(chat_id: &str) {
    info!("定时任务本轮开始: 同步文章");
    handler::sync::sync_articles(chat_id).await;

    info!("定时任务: 开始总结待总结文章");
    loop {
        match handler::summary::summarize_latest(chat_id).await {
            handler::summary::SummarizeOutcome::Done => {}
            outcome => {
                // 无待总结或某篇失败: 结束本轮, 避免对同一篇无限重试;
                // 失败篇仍保持「待总结」, 下一轮会自动再次尝试
                log::debug!("定时任务结束总结: {outcome:?}");
                break;
            }
        }
    }
    info!("定时任务本轮完成");
}

/// 解析 "HH:MM,HH:MM" 为升序去重的一天内时刻(距 0 点的分钟数)
fn parse_schedule(spec: &str) -> Option<Vec<u32>> {
    let mut times: Vec<u32> = Vec::new();
    for part in spec.split(',') {
        let part = part.trim();
        let (h, m) = part.split_once(':')?;
        let h: u32 = h.trim().parse().ok()?;
        let m: u32 = m.trim().parse().ok()?;
        if h > 23 || m > 59 {
            return None;
        }
        times.push(h * 60 + m);
    }
    if times.is_empty() {
        return None;
    }
    times.sort_unstable();
    times.dedup();
    Some(times)
}

/// 计算 now(epoch 秒)之后的下一个执行时刻(epoch 秒), 按北京时间(UTC+8)
///
/// 若 now 恰好落在某个时刻点所在分钟内(如 08:00:30), 视为该点已过, 顺延到下一个时刻点。
fn next_run_at(schedule: &[u32], now: u64) -> u64 {
    const BEIJING_OFFSET: u64 = 8 * 3600;
    let shifted = now + BEIJING_OFFSET;
    let day = shifted / 86400; // 北京时间下的日序号
    let minute = ((shifted % 86400) / 60) as u32;

    for &m in schedule {
        if m > minute {
            return day * 86400 + m as u64 * 60 - BEIJING_OFFSET;
        }
    }
    // 今天时刻点已全部过去, 顺延到明天第一个时刻点
    (day + 1) * 86400 + schedule[0] as u64 * 60 - BEIJING_OFFSET
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_secs()
}

fn fmt_schedule(schedule: &[u32]) -> String {
    schedule
        .iter()
        .map(|&m| fmt_minutes(m))
        .collect::<Vec<_>>()
        .join(", ")
}

fn fmt_minutes(minutes: u32) -> String {
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

/// epoch 秒 -> 北京时间 HH:MM
fn fmt_beijing_time(epoch: u64) -> String {
    fmt_minutes(((epoch + 8 * 3600) % 86400 / 60) as u32)
}

fn fmt_duration(secs: u64) -> String {
    let h = secs / 3600;
    let m = secs % 3600 / 60;
    if h > 0 {
        format!("{h} 小时 {m} 分")
    } else {
        format!("{m} 分")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_standard_schedule() {
        assert_eq!(parse_schedule("08:00,20:00"), Some(vec![480, 1200]));
    }

    #[test]
    fn parse_tolerates_space_and_dup() {
        assert_eq!(parse_schedule(" 8:00 , 20:00,08:00 "), Some(vec![480, 1200]));
    }

    #[test]
    fn parse_rejects_bad_input() {
        assert_eq!(parse_schedule(""), None);
        assert_eq!(parse_schedule("24:00"), None);
        assert_eq!(parse_schedule("08:60"), None);
        assert_eq!(parse_schedule("abc"), None);
    }

    /// 北京时间 2024-01-01 00:00 对应的 epoch 秒
    const BJ_MIDNIGHT: u64 = 1704067200 - 8 * 3600;

    #[test]
    fn next_run_same_day() {
        // 北京时间 06:00 -> 下一个 08:00
        let now = BJ_MIDNIGHT + 6 * 3600;
        assert_eq!(next_run_at(&[480, 1200], now), BJ_MIDNIGHT + 8 * 3600);
    }

    #[test]
    fn next_run_wraps_to_tomorrow() {
        // 北京时间 21:00 -> 明天 08:00
        let now = BJ_MIDNIGHT + 21 * 3600;
        assert_eq!(next_run_at(&[480, 1200], now), BJ_MIDNIGHT + 32 * 3600);
    }

    #[test]
    fn next_run_at_exact_minute_defers_to_next_slot() {
        // 恰好 08:00:00 -> 08:00 视为已过, 下一次是同一天 20:00
        let now = BJ_MIDNIGHT + 8 * 3600;
        assert_eq!(next_run_at(&[480, 1200], now), BJ_MIDNIGHT + 20 * 3600);
    }

    #[test]
    fn next_run_single_slot_midnight_edge() {
        // 只有 00:00 一个时刻; 北京时间 23:00 -> 明天 00:00
        let now = BJ_MIDNIGHT + 23 * 3600;
        assert_eq!(next_run_at(&[0], now), BJ_MIDNIGHT + 24 * 3600);
    }
}