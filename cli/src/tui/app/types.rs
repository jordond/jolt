//! Core types and constants for the TUI application.

/// Minimum refresh rate in milliseconds.
pub const MIN_REFRESH_MS: u64 = 500;

/// Maximum refresh rate in milliseconds.
pub const MAX_REFRESH_MS: u64 = 10000;

/// Step size for refresh rate adjustments in milliseconds.
pub const REFRESH_STEP_MS: u64 = 500;

/// Actions that can be performed in the TUI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Quit,
    ToggleHelp,
    ToggleAbout,
    ToggleSettings,
    SelectNext,
    SelectPrevious,
    ToggleExpand,
    KillProcess,
    ConfirmKill,
    CancelKill,
    ToggleKillSignal,
    CycleAppearance,
    OpenThemePicker,
    CloseThemePicker,
    SelectTheme,
    TogglePreviewAppearance,
    ToggleGraphView,
    ToggleMerge,
    PageUp,
    PageDown,
    Home,
    End,
    ExitSelectionMode,
    CycleSortColumn,
    ToggleSortDirection,
    IncreaseRefreshRate,
    DecreaseRefreshRate,
    OpenThemeImporter,
    CloseThemeImporter,
    ImporterToggleSelect,
    ImporterPreview,
    ImporterImport,
    ImporterRefresh,
    ImporterToggleSearch,
    ImporterFilterChar(char),
    ImporterFilterBackspace,
    ImporterClearFilter,
    ToggleHistory,
    HistoryPrevPeriod,
    HistoryNextPeriod,
    SettingsToggleValue,
    SettingsIncrement,
    SettingsDecrement,
    ToggleBatteryDetails,
    None,
}

/// Time period for history data display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HistoryPeriod {
    #[default]
    Today,
    Week,
    Month,
    All,
}

impl HistoryPeriod {
    /// Returns the next period in the cycle.
    pub fn next(self) -> Self {
        match self {
            HistoryPeriod::Today => HistoryPeriod::Week,
            HistoryPeriod::Week => HistoryPeriod::Month,
            HistoryPeriod::Month => HistoryPeriod::All,
            HistoryPeriod::All => HistoryPeriod::Today,
        }
    }

    /// Returns the previous period in the cycle.
    pub fn prev(self) -> Self {
        match self {
            HistoryPeriod::Today => HistoryPeriod::All,
            HistoryPeriod::Week => HistoryPeriod::Today,
            HistoryPeriod::Month => HistoryPeriod::Week,
            HistoryPeriod::All => HistoryPeriod::Month,
        }
    }

    /// Returns the display label for this period.
    pub fn label(self) -> &'static str {
        match self {
            HistoryPeriod::Today => "Today",
            HistoryPeriod::Week => "Week",
            HistoryPeriod::Month => "Month",
            HistoryPeriod::All => "All",
        }
    }

    /// Returns the number of days this period spans.
    pub fn days(self) -> u32 {
        match self {
            HistoryPeriod::Today => 1,
            HistoryPeriod::Week => 7,
            HistoryPeriod::Month => 30,
            HistoryPeriod::All => 365,
        }
    }
}

/// Column used for sorting the process list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortColumn {
    Pid,
    Name,
    Cpu,
    Memory,
    #[default]
    Energy,
}

impl SortColumn {
    /// Returns the next column in the cycle.
    pub fn next(self) -> Self {
        match self {
            SortColumn::Pid => SortColumn::Name,
            SortColumn::Name => SortColumn::Cpu,
            SortColumn::Cpu => SortColumn::Memory,
            SortColumn::Memory => SortColumn::Energy,
            SortColumn::Energy => SortColumn::Pid,
        }
    }
}

/// Current view/screen of the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppView {
    Main,
    Help,
    About,
    KillConfirm,
    ThemePicker,
    ThemeImporter,
    History,
    Settings,
    BatteryDetails,
}

#[cfg(test)]
mod tests {
    use super::*;

    // HistoryPeriod tests
    #[test]
    fn history_period_next_cycles_through_all_variants() {
        assert_eq!(HistoryPeriod::Today.next(), HistoryPeriod::Week);
        assert_eq!(HistoryPeriod::Week.next(), HistoryPeriod::Month);
        assert_eq!(HistoryPeriod::Month.next(), HistoryPeriod::All);
        assert_eq!(HistoryPeriod::All.next(), HistoryPeriod::Today);
    }

    #[test]
    fn history_period_prev_cycles_through_all_variants() {
        assert_eq!(HistoryPeriod::Today.prev(), HistoryPeriod::All);
        assert_eq!(HistoryPeriod::Week.prev(), HistoryPeriod::Today);
        assert_eq!(HistoryPeriod::Month.prev(), HistoryPeriod::Week);
        assert_eq!(HistoryPeriod::All.prev(), HistoryPeriod::Month);
    }

    #[test]
    fn history_period_label_returns_display_string() {
        assert_eq!(HistoryPeriod::Today.label(), "Today");
        assert_eq!(HistoryPeriod::Week.label(), "Week");
        assert_eq!(HistoryPeriod::Month.label(), "Month");
        assert_eq!(HistoryPeriod::All.label(), "All");
    }

    #[test]
    fn history_period_days_returns_expected_values() {
        assert_eq!(HistoryPeriod::Today.days(), 1);
        assert_eq!(HistoryPeriod::Week.days(), 7);
        assert_eq!(HistoryPeriod::Month.days(), 30);
        assert_eq!(HistoryPeriod::All.days(), 365);
    }

    #[test]
    fn history_period_default_is_today() {
        assert_eq!(HistoryPeriod::default(), HistoryPeriod::Today);
    }

    #[test]
    fn history_period_next_then_prev_returns_original() {
        assert_eq!(HistoryPeriod::Today.next().prev(), HistoryPeriod::Today);
        assert_eq!(HistoryPeriod::Week.next().prev(), HistoryPeriod::Week);
        assert_eq!(HistoryPeriod::Month.next().prev(), HistoryPeriod::Month);
        assert_eq!(HistoryPeriod::All.next().prev(), HistoryPeriod::All);
    }

    #[test]
    fn history_period_prev_then_next_returns_original() {
        assert_eq!(HistoryPeriod::Today.prev().next(), HistoryPeriod::Today);
        assert_eq!(HistoryPeriod::Week.prev().next(), HistoryPeriod::Week);
        assert_eq!(HistoryPeriod::Month.prev().next(), HistoryPeriod::Month);
        assert_eq!(HistoryPeriod::All.prev().next(), HistoryPeriod::All);
    }

    #[test]
    fn history_period_full_cycle_returns_to_start() {
        let start = HistoryPeriod::Today;
        let result = start.next().next().next().next();
        assert_eq!(result, start);
    }

    #[test]
    fn history_period_days_are_increasing() {
        assert!(HistoryPeriod::Today.days() < HistoryPeriod::Week.days());
        assert!(HistoryPeriod::Week.days() < HistoryPeriod::Month.days());
        assert!(HistoryPeriod::Month.days() < HistoryPeriod::All.days());
    }

    // SortColumn tests
    #[test]
    fn sort_column_next_cycles_through_all_variants() {
        assert_eq!(SortColumn::Pid.next(), SortColumn::Name);
        assert_eq!(SortColumn::Name.next(), SortColumn::Cpu);
        assert_eq!(SortColumn::Cpu.next(), SortColumn::Memory);
        assert_eq!(SortColumn::Memory.next(), SortColumn::Energy);
        assert_eq!(SortColumn::Energy.next(), SortColumn::Pid);
    }

    #[test]
    fn sort_column_default_is_energy() {
        assert_eq!(SortColumn::default(), SortColumn::Energy);
    }

    #[test]
    fn sort_column_full_cycle_returns_to_start() {
        let start = SortColumn::Pid;
        let result = start.next().next().next().next().next();
        assert_eq!(result, start);
    }

    #[test]
    fn sort_column_all_variants_are_distinct() {
        assert_ne!(SortColumn::Pid, SortColumn::Name);
        assert_ne!(SortColumn::Pid, SortColumn::Cpu);
        assert_ne!(SortColumn::Pid, SortColumn::Memory);
        assert_ne!(SortColumn::Pid, SortColumn::Energy);
        assert_ne!(SortColumn::Name, SortColumn::Cpu);
        assert_ne!(SortColumn::Name, SortColumn::Memory);
        assert_ne!(SortColumn::Name, SortColumn::Energy);
        assert_ne!(SortColumn::Cpu, SortColumn::Memory);
        assert_ne!(SortColumn::Cpu, SortColumn::Energy);
        assert_ne!(SortColumn::Memory, SortColumn::Energy);
    }

    // Action tests
    #[test]
    fn action_enum_has_none_variant() {
        let action = Action::None;
        assert_eq!(action, Action::None);
    }

    #[test]
    fn action_quit_is_distinct_from_none() {
        assert_ne!(Action::Quit, Action::None);
    }

    #[test]
    fn action_clone_produces_equal_value() {
        let action = Action::ToggleHelp;
        let cloned = action.clone();
        assert_eq!(action, cloned);
    }

    #[test]
    fn action_importer_filter_char_holds_character() {
        let action = Action::ImporterFilterChar('a');
        if let Action::ImporterFilterChar(c) = action {
            assert_eq!(c, 'a');
        } else {
            panic!("Expected ImporterFilterChar variant");
        }
    }

    #[test]
    fn action_importer_filter_char_different_chars_are_different() {
        assert_ne!(
            Action::ImporterFilterChar('a'),
            Action::ImporterFilterChar('b')
        );
    }

    #[test]
    fn action_navigation_variants_are_distinct() {
        assert_ne!(Action::SelectNext, Action::SelectPrevious);
        assert_ne!(Action::PageUp, Action::PageDown);
        assert_ne!(Action::Home, Action::End);
    }

    // AppView tests
    #[test]
    fn app_view_main_is_distinct_from_others() {
        assert_ne!(AppView::Main, AppView::Help);
        assert_ne!(AppView::Main, AppView::Settings);
        assert_ne!(AppView::Main, AppView::History);
    }

    #[test]
    fn app_view_all_variants_are_distinct() {
        assert_ne!(AppView::Main, AppView::Help);
        assert_ne!(AppView::Main, AppView::About);
        assert_ne!(AppView::Main, AppView::KillConfirm);
        assert_ne!(AppView::Main, AppView::ThemePicker);
        assert_ne!(AppView::Main, AppView::ThemeImporter);
        assert_ne!(AppView::Main, AppView::History);
        assert_ne!(AppView::Main, AppView::Settings);
        assert_ne!(AppView::Main, AppView::BatteryDetails);
    }

    #[test]
    fn app_view_clone_produces_equal_value() {
        let view = AppView::Settings;
        let cloned = view;
        assert_eq!(view, cloned);
    }

    #[test]
    fn app_view_copy_is_implemented() {
        let view = AppView::History;
        let copied: AppView = view;
        assert_eq!(view, copied);
    }

    // Constants tests
    #[test]
    fn refresh_constants_have_valid_range() {
        const { assert!(MIN_REFRESH_MS < MAX_REFRESH_MS) };
        const { assert!(REFRESH_STEP_MS > 0) };
        const { assert!(MIN_REFRESH_MS > 0) };
    }

    #[test]
    fn refresh_step_divides_range_evenly() {
        // Verify that stepping from min to max is possible
        assert_eq!((MAX_REFRESH_MS - MIN_REFRESH_MS) % REFRESH_STEP_MS, 0);
    }
}
