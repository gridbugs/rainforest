use chargrid::{prelude::*, text::StyledString};
use rainforest_game::{CellVisibility, Game, Tile};

#[derive(Clone, Copy, Debug)]
enum MessageVerb {
    See,
    Remember,
}

pub fn examine(game: &Game, coord: Coord) -> Option<StyledString> {
    let vis_count = game.visibility_grid().count();
    let mut entity_under_cursor = None;
    if let Some(visibility_cell_under_cursor) = game.visibility_grid().get_cell(coord) {
        let verb = match visibility_cell_under_cursor.visibility(vis_count) {
            CellVisibility::CurrentlyVisibleWithLightColour(Some(_)) => Some(MessageVerb::See),
            CellVisibility::PreviouslyVisible => Some(MessageVerb::Remember),
            CellVisibility::NeverVisible
            | CellVisibility::CurrentlyVisibleWithLightColour(None) => None,
        };
        if let Some(verb) = verb {
            if let Some(floor) = visibility_cell_under_cursor.tile_layers().floor {
                entity_under_cursor = Some((floor.tile, verb));
            }
            if let Some(feature) = visibility_cell_under_cursor.tile_layers().feature {
                entity_under_cursor = Some((feature.tile, verb));
            }
            if let Some(character) = visibility_cell_under_cursor.tile_layers().character {
                entity_under_cursor = Some((character.tile, verb));
            }
            if let Some(item) = visibility_cell_under_cursor.tile_layers().item {
                entity_under_cursor = Some((item.tile, verb));
            }
        }
    }
    entity_under_cursor.and_then(|(tile, verb)| {
        tile_str(tile).map(|label| match label {
            TileLabel::Name(name) => {
                let verb_str = match verb {
                    MessageVerb::Remember => "remember seeing",
                    MessageVerb::See => "see",
                };
                StyledString::plain_text(format!("You {} {} here.", verb_str, name))
            }
            TileLabel::Literal(literal) => StyledString::plain_text(literal.to_string()),
        })
    })
}

enum TileLabel {
    Literal(&'static str),
    Name(&'static str),
}

fn tile_str(tile: Tile) -> Option<TileLabel> {
    let label = match tile {
        Tile::Player => TileLabel::Literal("It's you!"),
        Tile::DoorClosed(_) | Tile::DoorOpen(_) => TileLabel::Name("a door"),
        Tile::Wall | Tile::RuinsWall => TileLabel::Name("a wall"),
        Tile::BulletinBoard => TileLabel::Name("a bulletin board"),
        Tile::Floor | Tile::RuinsFloor => TileLabel::Name("the floor"),
        Tile::Ground => TileLabel::Name("the ground"),
        Tile::Window(_) => TileLabel::Name("a window"),
        Tile::Tree0 | Tile::Tree1 | Tile::Tree2 => TileLabel::Name("a tree"),
        Tile::Water => TileLabel::Name("water"),
        Tile::Altar => TileLabel::Name("an altar"),
        Tile::Lamp => TileLabel::Name("a lamp"),
    };
    Some(label)
}
