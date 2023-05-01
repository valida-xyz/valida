use p3_air::types::AirTypes;
use p3_air::window::AirWindow;
use p3_air::Air;

pub trait Chip<T, W, CC>: Air<T, W>
where
    T: AirTypes,
    W: AirWindow<T>,
{
}
