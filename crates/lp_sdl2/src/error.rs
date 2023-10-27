use sdl2::video::WindowBuildError;
use sdl2::ttf::FontError;
use sdl2::IntegerOrSdlError;
use thiserror::Error;
use derive_more::Display;

#[derive(Debug, Display, Clone, Copy)]
pub enum Subsystem {
    #[display(fmt = "core")]
    Core,
    #[display(fmt = "video")]
    Video,
    #[display(fmt = "event pump")]
    EventPump,
    #[display(fmt = "ttf")]
    Ttf,
}

#[derive(Debug, Display, Clone, Copy)]
pub enum AssetKind {
    #[display(fmt = "font")]
    Font,
    #[display(fmt = "image")]
    Image,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to initialize SDL feature '{subsystem}'. SDL provided an explanation: {explanation}")]
    SystemInit {
        subsystem: Subsystem,
        explanation: String,
    },
    #[error("Failed to initialize the SDL window: {0}")]
    WindowInit(ReducedSdlError),
    #[error("Error while drawing: {0}")]
    Draw(String),
    #[error("Failed to load asset of type '{kind}'. SDL provided an explanation: {explanation}")]
    AssetLoad {
        kind: AssetKind,
        explanation: String,
    },
}

impl Error {
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Display, Debug)]
#[display(fmt = "{content}")]
pub struct ReducedSdlError {
    content: String,
}

impl From<WindowBuildError> for ReducedSdlError {
    fn from(value: WindowBuildError) -> Self {
        use WindowBuildError::*;
        <Self as From<String>>::from(match value {
            SdlError(s) => s,
            InvalidTitle(s) => format!("invalid window title ({s})"),
            HeightOverflows(n) => format!("height too large ({n})"),
            WidthOverflows(n) => format!("width too large ({n})"),
        })
    }
}

impl From<IntegerOrSdlError> for ReducedSdlError {
    fn from(value: IntegerOrSdlError) -> Self {
        use IntegerOrSdlError::*;
        <Self as From<String>>::from(match value {
            SdlError(s) => s,
            IntegerOverflows(reason, n) => format!("value overflowed ({n}): {reason}"),
        })
    }
}

impl From<FontError> for ReducedSdlError {
    fn from(value: FontError) -> Self {
        Self { content: value.to_string() }
    }
}

impl From<String> for ReducedSdlError {
    fn from(value: String) -> Self {
        Self { content: value }
    }
}

pub(crate) fn system_ttf(explanation: impl ToString) -> Error {
    Error::SystemInit {
        subsystem: Subsystem::Ttf,
        explanation: explanation.to_string(),
    }
}

pub(crate) fn system_video(explanation: impl ToString) -> Error {
    Error::SystemInit {
        subsystem: Subsystem::Video,
        explanation: explanation.to_string(),
    }
}

pub(crate) fn system_core(explanation: impl ToString) -> Error {
    Error::SystemInit {
        subsystem: Subsystem::Core,
        explanation: explanation.to_string(),
    }
}

pub(crate) fn system_event_pump(explanation: impl ToString) -> Error {
    Error::SystemInit {
        subsystem: Subsystem::EventPump,
        explanation: explanation.to_string(),
    }
}

pub(crate) fn asset_font(explanation: impl ToString) -> Error {
    Error::AssetLoad {
        kind: AssetKind::Font,
        explanation: explanation.to_string(),
    }
}

pub(crate) fn window_init(inner: impl Into<ReducedSdlError>) -> Error {
    Error::WindowInit(inner.into())
}

pub(crate) fn draw(inner: impl ToString) -> Error {
    Error::Draw(inner.to_string())
}

