use bevy::{
    ecs::schedule::{GraphInfo, Schedulable, ScheduleConfigs},
    time::common_conditions::on_timer,
};

use crate::prelude::*;

pub fn on_turn_timer() -> impl FnMut(Res<Time>) -> bool + Clone {
    on_timer(ms(500))
}

pub trait SchedulableExt<
    S: Schedulable<Metadata = GraphInfo, GroupMetadata = bevy::ecs::schedule::Chain>,
    Marker,
>
{
    fn run_on_turn_timer(self) -> ScheduleConfigs<S>;
}
impl<
    T: Schedulable<Metadata = GraphInfo, GroupMetadata = bevy::ecs::schedule::Chain>,
    Marker,
    I: IntoScheduleConfigs<T, Marker>,
> SchedulableExt<T, Marker> for I
{
    fn run_on_turn_timer(self) -> ScheduleConfigs<T> {
        self.run_if(on_turn_timer())
    }
}
