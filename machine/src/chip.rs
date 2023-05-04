use p3_air::{Air, AirBuilder};

pub trait Chip<AB>: Air<AB>
where
    AB: AirBuilder,
{
}
