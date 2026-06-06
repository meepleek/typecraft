use std::str::FromStr;

use crate::prelude::*;
use bevy::math::U16Vec2;

#[derive(Debug, PartialEq)]
pub struct TemplateTile {
    pub tile: Coords,
    pub kind: TemplateTileKind,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TemplateEnemy {
    Wallie,
    Bouncer(TileDir),
}

#[derive(Debug, PartialEq)]
pub enum TemplateTileKind {
    PermaWall,
    Empty,
    Wall,
    Player,
    Goal,
    Enemy(TemplateEnemy),
}

impl TryFrom<char> for TemplateTileKind {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        use TileDiagDir::*;
        use TileDir::*;
        use TileOrthoDir::*;

        match value {
            '#' => Ok(Self::PermaWall),
            '.' => Ok(Self::Empty),
            'W' => Ok(Self::Wall),
            '@' => Ok(Self::Player),
            'G' => Ok(Self::Goal),
            '*' => Ok(Self::Enemy(TemplateEnemy::Wallie)),
            '>' => Ok(Self::Enemy(TemplateEnemy::Bouncer(Ortho(East)))),
            '<' => Ok(Self::Enemy(TemplateEnemy::Bouncer(Ortho(West)))),
            '^' => Ok(Self::Enemy(TemplateEnemy::Bouncer(Ortho(North)))),
            'v' => Ok(Self::Enemy(TemplateEnemy::Bouncer(Ortho(South)))),
            '/' => Ok(Self::Enemy(TemplateEnemy::Bouncer(Diag(SouthEast)))),
            '\\' => Ok(Self::Enemy(TemplateEnemy::Bouncer(Diag(SouthWest)))),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct GridChunkTemplate {
    pub grid_size: U16Vec2,
    pub tiles: Vec<TemplateTile>,
    pub player: Coords,
    pub goal: Coords,
}

impl FromStr for GridChunkTemplate {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut size = Coords::ZERO;
        let mut errors = Vec::new();
        let mut tiles = Vec::with_capacity(s.len());
        let mut player = Vec::with_capacity(1);
        let mut goal = Vec::with_capacity(1);

        for (tile, c) in s
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .enumerate()
            .flat_map(|(y, line)| {
                line.chars()
                    .enumerate()
                    .map(move |(x, c)| (Coords::new(x as i16, y as i16), c))
            })
        {
            size = size.max(tile);
            match TemplateTileKind::try_from(c) {
                Ok(kind) => {
                    match kind {
                        TemplateTileKind::Player => player.push(tile),
                        TemplateTileKind::Goal => goal.push(tile),
                        _ => {}
                    }

                    tiles.push(TemplateTile { tile, kind });
                }
                Err(_) => {
                    errors.push(format!("Invalid char '{c}' at {tile}"));
                }
            }
        }

        if player.is_empty() {
            errors.push("No player".to_string());
        }
        if player.len() > 1 {
            errors.push("Multiple players".to_string());
        }
        if goal.is_empty() {
            errors.push("No goal".to_string());
        }
        if goal.len() > 1 {
            errors.push("Multiple goals".to_string());
        }

        if errors.is_empty() {
            Ok(GridChunkTemplate {
                grid_size: (size + Coords::ONE).as_u16vec2(),
                tiles,
                player: player.pop().unwrap(),
                goal: goal.pop().unwrap(),
            })
        } else {
            Err(errors.join("\n"))
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use test_case::test_case;
    use tracing_test::traced_test;

    use super::*;
    use TemplateTileKind::*;

    #[test]
    #[traced_test]
    fn parse_ok() {
        let lvl = "
##.*WW
#@..WG
";
        let parsed_template = lvl.parse::<GridChunkTemplate>();

        assert_eq!(
            Ok(GridChunkTemplate {
                grid_size: U16Vec2::new(6, 2),
                tiles: [
                    ((0, 0), PermaWall),
                    ((1, 0), PermaWall),
                    ((2, 0), Empty),
                    ((3, 0), Enemy(TemplateEnemy::Wallie)),
                    ((4, 0), Wall),
                    ((5, 0), Wall),
                    ((0, 1), PermaWall),
                    ((1, 1), Player),
                    ((2, 1), Empty),
                    ((3, 1), Empty),
                    ((4, 1), Wall),
                    ((5, 1), Goal),
                ]
                .into_iter()
                .map(|(tile, kind)| TemplateTile {
                    tile: tile.into(),
                    kind
                })
                .collect(),
                player: Coords::new(1, 1),
                goal: Coords::new(5, 1),
            }),
            parsed_template
        );
    }

    #[test]
    #[traced_test]
    fn parse_ok_6x5() {
        let lvl = "
###WGW
...WWW
......
.@....
......
";
        let template = lvl.parse::<GridChunkTemplate>();
        pretty_assertions::assert_eq!(
            Ok(GridChunkTemplate {
                grid_size: U16Vec2::new(6, 5),
                tiles: [
                    ((0, 0), TemplateTileKind::PermaWall),
                    ((1, 0), TemplateTileKind::PermaWall),
                    ((2, 0), TemplateTileKind::PermaWall),
                    ((3, 0), TemplateTileKind::Wall),
                    ((4, 0), TemplateTileKind::Goal),
                    ((5, 0), TemplateTileKind::Wall),
                    ((0, 1), TemplateTileKind::Empty),
                    ((1, 1), TemplateTileKind::Empty),
                    ((2, 1), TemplateTileKind::Empty),
                    ((3, 1), TemplateTileKind::Wall),
                    ((4, 1), TemplateTileKind::Wall),
                    ((5, 1), TemplateTileKind::Wall),
                    ((0, 2), TemplateTileKind::Empty),
                    ((1, 2), TemplateTileKind::Empty),
                    ((2, 2), TemplateTileKind::Empty),
                    ((3, 2), TemplateTileKind::Empty),
                    ((4, 2), TemplateTileKind::Empty),
                    ((5, 2), TemplateTileKind::Empty),
                    ((0, 3), TemplateTileKind::Empty),
                    ((1, 3), TemplateTileKind::Player),
                    ((2, 3), TemplateTileKind::Empty),
                    ((3, 3), TemplateTileKind::Empty),
                    ((4, 3), TemplateTileKind::Empty),
                    ((5, 3), TemplateTileKind::Empty),
                    ((0, 4), TemplateTileKind::Empty),
                    ((1, 4), TemplateTileKind::Empty),
                    ((2, 4), TemplateTileKind::Empty),
                    ((3, 4), TemplateTileKind::Empty),
                    ((4, 4), TemplateTileKind::Empty),
                    ((5, 4), TemplateTileKind::Empty),
                ]
                .into_iter()
                .map(|(tile, kind)| template::TemplateTile {
                    tile: tile.into(),
                    kind
                })
                .collect(),
                player: Coords::new(1, 3),
                goal: Coords::new(4, 0),
            }),
            template,
        );
    }

    #[test]
    #[traced_test]
    fn parse_ok_whitespace_trimmed() {
        let lvl = " \t @.G  ";
        let parsed_template = lvl.parse::<GridChunkTemplate>();

        assert_eq!(
            Ok(GridChunkTemplate {
                grid_size: U16Vec2::new(3, 1),
                tiles: [((0, 0), Player), ((1, 0), Empty), ((2, 0), Goal),]
                    .into_iter()
                    .map(|(tile, kind)| TemplateTile {
                        tile: tile.into(),
                        kind
                    })
                    .collect(),
                player: Coords::new(0, 0),
                goal: Coords::new(2, 0),
            }),
            parsed_template
        );
    }

    #[test_case("..G", "No player")]
    #[test_case("@@.G", "Multiple players")]
    #[test_case("@..", "No goal")]
    #[test_case("@.GG", "Multiple goals")]
    #[test_case("@.$._.G", "Invalid char '$' at [2, 0]\nInvalid char '_' at [4, 0]")]
    #[traced_test]
    fn parse_missing_objects(lvl: &str, err_msg: &str) {
        let parsed_template = lvl.parse::<GridChunkTemplate>();

        assert_eq!(Err(err_msg.to_string()), parsed_template);
    }
}
