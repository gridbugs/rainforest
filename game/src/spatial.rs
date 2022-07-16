gridbugs::spatial_table::declare_layers_module! {
    layers {
        floor: Floor,
        feature: Feature,
        character: Character,
        item: Item,
    }
}
pub use layers::{Layer, Layers};
pub type SpatialTable = gridbugs::spatial_table::SpatialTable<Layers>;
pub type Location = gridbugs::spatial_table::Location<Layer>;
pub use gridbugs::spatial_table::UpdateError;
