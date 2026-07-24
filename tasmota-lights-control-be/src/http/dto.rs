use crate::application::{
    bulb_service::BulbService, control_service::ControlService,
    direct_link_service::DirectLinkService, profile_service::ProfileService,
    settings_service::SettingsService, status_service::StatusService,
};
use std::sync::{Arc, atomic::AtomicBool};
#[derive(Clone)]
pub struct AppState {
    pub bulbs: Arc<BulbService>,
    pub profiles: Arc<ProfileService>,
    pub control: Arc<ControlService>,
    pub status: Arc<StatusService>,
    pub settings: Arc<SettingsService>,
    pub direct_links: Arc<DirectLinkService>,
    pub shutting_down: Arc<AtomicBool>,
}
