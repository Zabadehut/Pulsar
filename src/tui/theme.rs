use ratatui::style::{Color, Modifier, Style};

pub struct Theme {
    pub primary: Color,
    pub accent: Color,
    pub alert: Color,
    pub neutral: Color,
    #[allow(dead_code)]
    pub background: Color,
    pub text: Color,
    pub border: Color,
}

impl Theme {
    pub const NAMES: [&str; 3] = ["dark", "light", "cosmic"];

    pub fn dark() -> Self {
        Self {
            primary: Color::Rgb(123, 47, 255), // #7B2FFF
            accent: Color::Rgb(0, 255, 178),   // #00FFB2
            alert: Color::Rgb(255, 77, 109),   // #FF4D6D
            neutral: Color::Rgb(200, 200, 212),
            background: Color::Rgb(8, 8, 16),
            text: Color::White,
            border: Color::Rgb(60, 60, 90),
        }
    }

    pub fn light() -> Self {
        Self {
            primary: Color::Rgb(90, 20, 200),
            accent: Color::Rgb(0, 180, 130),
            alert: Color::Rgb(200, 40, 80),
            neutral: Color::DarkGray,
            background: Color::White,
            text: Color::Black,
            border: Color::Gray,
        }
    }

    pub fn cosmic() -> Self {
        Self {
            primary: Color::Rgb(255, 140, 66),
            accent: Color::Rgb(72, 191, 227),
            alert: Color::Rgb(217, 59, 86),
            neutral: Color::Rgb(214, 211, 201),
            background: Color::Rgb(16, 18, 28),
            text: Color::Rgb(245, 240, 230),
            border: Color::Rgb(110, 98, 140),
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name {
            "light" => Self::light(),
            "cosmic" => Self::cosmic(),
            _ => Self::dark(),
        }
    }

    pub fn normalize_name(name: &str) -> &'static str {
        match name {
            "light" => "light",
            "cosmic" => "cosmic",
            _ => "dark",
        }
    }

    pub fn next_name(name: &str) -> &'static str {
        let current = Self::normalize_name(name);
        let index = Self::NAMES
            .iter()
            .position(|candidate| *candidate == current)
            .unwrap_or(0);
        Self::NAMES[(index + 1) % Self::NAMES.len()]
    }

    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    pub fn highlight_style(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.neutral)
    }

    pub fn alert_style(&self) -> Style {
        Style::default().fg(self.alert).add_modifier(Modifier::BOLD)
    }

    pub fn gauge_style(&self) -> Style {
        Style::default().fg(self.accent)
    }

    pub fn gauge_alert_style(&self) -> Style {
        Style::default().fg(self.alert)
    }

    /// Retourne le style de gauge selon le pourcentage (vert < 75%, rouge >= 90%)
    pub fn gauge_for_pct(&self, pct: f64) -> Style {
        if pct >= 90.0 {
            self.gauge_alert_style()
        } else if pct >= 75.0 {
            self.highlight_style()
        } else {
            self.gauge_style()
        }
    }
}
