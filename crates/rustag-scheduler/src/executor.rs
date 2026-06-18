//! The Activity Scheduler background loop.
//!
//! Once per second the loop reloads the stagenet's *enabled* schedules from the
//! store (so changes via the API/CLI take effect live), tracks each schedule's
//! next fire time in memory, and executes any that are due. Each firing's
//! outcome (signature or error) is written back via
//! [`AccountStore::record_schedule_run`], so the dashboard and `rustag schedule
//! list` show live run counts and last status.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use uuid::Uuid;

use rustag_core::{AccountStore, ScheduleRecord, Stagenet};

use crate::activity::Action;
use crate::error::Result;
use crate::schedule::Schedule;

/// How often the loop wakes to check for due activities.
const TICK: Duration = Duration::from_secs(1);

/// The Activity Scheduler.
pub struct Scheduler;

impl Scheduler {
    /// Spawn the scheduler loop for `stagenet`. The task runs until its
    /// [`JoinHandle`] is aborted.
    pub fn spawn(stagenet: Arc<RwLock<Stagenet>>, store: Arc<AccountStore>) -> JoinHandle<()> {
        tokio::spawn(async move {
            let stagenet_id = stagenet.read().await.id();
            tracing::info!(%stagenet_id, "activity scheduler started");
            let mut next_fire: HashMap<Uuid, DateTime<Utc>> = HashMap::new();
            let mut ticker = tokio::time::interval(TICK);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                ticker.tick().await;
                if let Err(e) = tick(&stagenet, &store, &stagenet_id, &mut next_fire).await {
                    tracing::warn!(error = %e, "scheduler tick failed");
                }
            }
        })
    }
}

async fn tick(
    stagenet: &Arc<RwLock<Stagenet>>,
    store: &Arc<AccountStore>,
    stagenet_id: &Uuid,
    next_fire: &mut HashMap<Uuid, DateTime<Utc>>,
) -> Result<()> {
    let now = Utc::now();
    let schedules = store.list_schedules(stagenet_id, true).await?;

    // Forget schedules that were disabled or deleted since the last tick.
    let live: HashSet<Uuid> = schedules.iter().map(|s| s.id).collect();
    next_fire.retain(|id, _| live.contains(id));

    for rec in &schedules {
        let sched = match Schedule::parse(&rec.schedule) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(activity = %rec.name, error = %e, "invalid schedule expression; skipping");
                continue;
            }
        };
        let fire_at = *next_fire
            .entry(rec.id)
            .or_insert_with(|| sched.next_after(now).unwrap_or_else(|| far_future(now)));

        if now < fire_at {
            continue;
        }
        fire(stagenet, store, rec).await;
        // Anchor the next fire to the *scheduled* time, not `now`, so intervals
        // don't drift by the execution duration. If the loop stalled, skip the
        // missed fires (advance past `now`) rather than building a backlog.
        let mut next = sched.next_after(fire_at).unwrap_or_else(|| far_future(now));
        while next <= now {
            next = sched.next_after(next).unwrap_or_else(|| far_future(now));
        }
        next_fire.insert(rec.id, next);
    }
    Ok(())
}

async fn fire(stagenet: &Arc<RwLock<Stagenet>>, store: &Arc<AccountStore>, rec: &ScheduleRecord) {
    let action: Action = match serde_json::from_str(&rec.action_json) {
        Ok(a) => a,
        Err(e) => {
            tracing::warn!(activity = %rec.name, error = %e, "invalid action JSON; skipping");
            let _ = store
                .record_schedule_run(&rec.id, &format!("invalid action: {e}"), None)
                .await;
            return;
        }
    };

    // Hold the write lock only for the execution itself.
    let result = {
        let mut sn = stagenet.write().await;
        action.execute(&mut sn).await
    };

    match result {
        Ok(sig) => {
            tracing::info!(activity = %rec.name, signature = ?sig, "scheduled activity fired");
            let _ = store
                .record_schedule_run(&rec.id, "ok", sig.as_deref())
                .await;
        }
        Err(e) => {
            tracing::warn!(activity = %rec.name, error = %e, "scheduled activity failed");
            let _ = store
                .record_schedule_run(&rec.id, &e.to_string(), None)
                .await;
        }
    }
}

fn far_future(now: DateTime<Utc>) -> DateTime<Utc> {
    now + chrono::Duration::days(3650)
}
