use p3_field::packed::PackedField;

pub struct AirWindow<'a, P: PackedField> {
    pub local_row: &'a [P],
    pub next_row: &'a [P],
}
