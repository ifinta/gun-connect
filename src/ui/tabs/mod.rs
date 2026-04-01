pub mod info;
pub mod settings;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Tab {
    #[default]
    Info,
    Settings,
}
