pub mod home;
pub mod zsozso;
pub mod networking;
pub mod info;
pub mod settings;
pub mod log;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Tab {
    #[default]
    Cyf,
    Zsozso,
    Networking,
    Info,
    Settings,
    Log,
}
