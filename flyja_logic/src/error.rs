#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum FlyjaError {
    #[error("Element not found")]
    ElementNotFound,
    #[error("This drag is illegal, size be minus")]
    DragIllegal,
}
