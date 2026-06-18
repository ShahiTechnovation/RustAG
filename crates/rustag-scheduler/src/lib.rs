//! RustAG Activity Scheduler.
//!
//! Schedule recurring on-chain actions against a running stagenet to simulate
//! realistic, ongoing usage — periodic swaps, deposits, liquidations, faucet
//! top-ups. An *activity* pairs a [`Schedule`] (a `@every` interval or a 5-field
//! cron expression) with an [`Action`] (airdrop, signed transfer, or replay of a
//! pre-signed transaction).
//!
//! ```no_run
//! # async fn demo() -> Result<(), Box<dyn std::error::Error>> {
//! use std::sync::Arc;
//! use tokio::sync::RwLock;
//! use rustag_core::Stagenet;
//! use rustag_scheduler::{register_activity, Action, Scheduler};
//!
//! let sn = Stagenet::local("demo").await?;
//! let store = sn.store();
//! let id = sn.id();
//! let stagenet = Arc::new(RwLock::new(sn));
//!
//! // Top up a faucet wallet with 1 SOL every 30 seconds.
//! register_activity(
//!     &store, id, "faucet", "@every 30s",
//!     &Action::Airdrop { pubkey: "<WALLET>".into(), sol: 1.0 },
//! ).await?;
//!
//! Scheduler::spawn(stagenet, store);
//! # Ok(()) }
//! ```

mod activity;
mod error;
mod executor;
mod schedule;

pub use activity::Action;
pub use error::{Result, SchedulerError};
pub use executor::Scheduler;
pub use schedule::{CronSchedule, Schedule};

use uuid::Uuid;

use rustag_core::{AccountStore, ScheduleRecord};

/// Validate and persist a new activity, returning its stored record.
///
/// Both the schedule expression and the action are validated *before* anything
/// is written, so the scheduler loop only ever sees well-formed activities.
pub async fn register_activity(
    store: &AccountStore,
    stagenet_id: Uuid,
    name: &str,
    schedule: &str,
    action: &Action,
) -> Result<ScheduleRecord> {
    Schedule::parse(schedule)?; // reject bad cron/interval up front
    action.validate()?; // reject bad pubkeys/keys/blobs up front

    let rec = ScheduleRecord {
        id: Uuid::new_v4(),
        stagenet_id,
        name: name.to_string(),
        schedule: schedule.to_string(),
        action_json: serde_json::to_string(action)?,
        enabled: true,
        run_count: 0,
        last_run: None,
        last_status: None,
        last_signature: None,
        created_at: chrono::Utc::now(),
    };
    store.upsert_schedule(&rec).await?;
    Ok(rec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustag_core::Stagenet;
    use solana_pubkey::Pubkey;

    #[tokio::test]
    async fn register_validates_and_persists() {
        let sn = Stagenet::local("sched-reg").await.unwrap();
        let store = sn.store();
        let id = sn.id();

        // Bad schedule is rejected before persisting.
        let action = Action::Airdrop {
            pubkey: Pubkey::new_unique().to_string(),
            sol: 1.0,
        };
        assert!(register_activity(&store, id, "x", "not-a-cron", &action)
            .await
            .is_err());

        // Good one persists and is listed.
        let rec = register_activity(&store, id, "faucet", "@every 10s", &action)
            .await
            .unwrap();
        assert_eq!(rec.name, "faucet");
        let listed = store.list_schedules(&id, true).await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, rec.id);
    }
}
