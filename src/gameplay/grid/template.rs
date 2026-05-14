use std::str::FromStr;

use bevy::math::U16Vec2;

use crate::prelude::*;

#[derive(Debug)]
pub struct TemplateTile {
    tile: Coords,
    kind: TemplateTileKind,
}

#[derive(Debug)]
pub enum TemplateTileKind {
    PermaWall,
    Empty,
    Wall,
    Player,
    Goal,
    // Enemy,
}
impl TemplateTileKind {
    pub const PERMAWALL: char = '#';
    pub const EMPTY: char = '.';
    pub const WALL: char = 'W';
    pub const PLAYER: char = '@';
    pub const GOAL: char = 'G';
    // pub const ENEMY: char = '#';
}

impl TryFrom<char> for TemplateTileKind {
    type Error = String;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            Self::PERMAWALL => Ok(Self::PermaWall),
            Self::EMPTY => Ok(Self::Empty),
            Self::WALL => Ok(Self::Wall),
            Self::PLAYER => Ok(Self::Player),
            Self::GOAL => Ok(Self::Goal),
            _ => Err(format!("Unknown tile char {value}")),
        }
    }
}

pub struct GridTemplate {
    size: U16Vec2,
    tiles: Vec<TemplateTile>,
    player: Coords,
    goal: Coords,
}
impl GridTemplate {
    pub fn spawn<'a>(self, e_cmd: &'a mut EntityCommands<'a>) -> &'a mut EntityCommands<'a> {
        e_cmd.with_children(|b| {
            let mut rng = rand::rng();
            let mut grid = grid::Grid::new(self.size.x, self.size.y);
            for tt in self.tiles {
                let pos = grid
                    .tile_to_world(tt.tile)
                    .expect(&format!("Invalid cords {}", tt.tile));
                let transform = Transform::from_translation(pos.extend(0.));
                let transform_scale0 = transform.clone().with_scale(Vec2::ZERO.extend(1.));
                let spawn_targetable_char = match tt.kind {
                    TemplateTileKind::PermaWall => None,
                    TemplateTileKind::Empty => Some(None),
                    TemplateTileKind::Wall => {
                        // todo: wall
                        Some(None)
                    }
                    TemplateTileKind::Player => {
                        let e = b.spawn((player::player(), transform_scale0.clone())).id();
                        grid.place_entity(
                            tile::TileObject {
                                entity: e,
                                kind: tile::TileObjectKind::Player,
                            },
                            tt.tile,
                        )
                        .expect("Failed to place tile object");

                        Some(Some(e))
                    }
                    TemplateTileKind::Goal => {
                        // todo: goal
                        Some(None)
                    }
                };
                if let Some(tween_e_src) = spawn_targetable_char {
                    const TWEEN_STEP_MS: u64 = 110;
                    let tile_dist_to_player = self.player.chebyshev_distance(tt.tile);
                    let tween_delay = ms(tile_dist_to_player as u64 * TWEEN_STEP_MS);
                    let neighbour_chars = grid.neighbour_chars(tt.tile);

                    let mut targetable_char_e = None;
                    for _ in 0..100 {
                        let c = grid
                            .move_chars
                            .iter()
                            .choose(&mut rng)
                            .expect("Failed to pick random move char");
                        if !neighbour_chars.contains(c) {
                            let (t, show_char) = if tween_e_src.is_some() {
                                (transform, false)
                            } else {
                                (transform_scale0, true)
                            };

                            let start_alpha = if show_char {
                                tile::TILE_ALPHA_TARGETABLE
                            } else {
                                tile::TILE_ALPHA_HIDDEN
                            };
                            let e = b
                                .spawn((
                                    t,
                                    Text2d::new(*c),
                                    TextFont::from_font_size(40.),
                                    TextColor(Color::WHITE.with_alpha(start_alpha)),
                                    tile::CharWiggle::new(
                                        ms(rng.random_range(0..5_000)),
                                        tile_dist_to_player,
                                    ),
                                ))
                                .id();
                            if show_char && tile_dist_to_player != 1 {
                                b.spawn(
                                    TextAlphaLensSrc::absolute(
                                        start_alpha,
                                        tile::TILE_ALPHA_INACTIVE,
                                    )
                                    .duration(ms(210))
                                    .delay(tween_delay + ms(150))
                                    .target(e),
                                );
                            }

                            grid.targetable_tiles.insert(
                                tt.tile,
                                tile::TargetableTile {
                                    move_char: *c,
                                    move_char_e: e,
                                },
                            );
                            targetable_char_e = Some(e);
                            break;
                        }
                    }

                    let tween_e = tween_e_src
                        .unwrap_or(targetable_char_e.expect("Failed to spawn targetable tile"));
                    b.spawn(
                        TransformScaleLensSrc::new(Vec2::ONE)
                            .duration(ms(400))
                            .target(tween_e)
                            .delay(tween_delay)
                            .easing(EaseFunction::BackOut),
                    );
                }
            }

            b.spawn((grid, Transform::default(), Visibility::default()));
        });
        e_cmd
    }
}

impl FromStr for GridTemplate {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut size = Coords::ZERO;
        let mut errors = Vec::new();
        let mut tiles = Vec::with_capacity(s.len());
        let mut player = None;
        let mut goal = None;

        for (tile, c) in s
            .lines()
            .filter(|l| !l.trim().is_empty())
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
                        TemplateTileKind::Player => player = Some(tile),
                        TemplateTileKind::Goal => goal = Some(tile),
                        _ => {}
                    }

                    tiles.push(TemplateTile { tile, kind });
                }
                Err(_) => {
                    errors.push(format!("Invalid char '{c}'"));
                }
            }
        }

        if player.is_none() {
            errors.push("No player".to_string());
        }
        if goal.is_none() {
            errors.push("No goal".to_string());
        }

        if errors.is_empty() {
            Ok(GridTemplate {
                size: (size + Coords::ONE).as_u16vec2(),
                tiles,
                player: player.unwrap(),
                goal: goal.unwrap(),
            })
        } else {
            Err(errors.join("\n"))
        }
    }
}
