/// Component library whose Storybook coverage is being inspected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Framework {
    /// React components and Component Story Format files.
    React,
    /// Vue single-file components and Component Story Format files.
    Vue,
    /// Angular components and Component Story Format files.
    Angular,
}

impl Framework {
    /// Human-readable framework name used in reports.
    pub const fn name(self) -> &'static str {
        match self {
            Self::React => "React",
            Self::Vue => "Vue",
            Self::Angular => "Angular",
        }
    }
}
