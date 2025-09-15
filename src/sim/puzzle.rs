use itertools::Itertools;
use rand::seq::IndexedRandom;

use super::elements::{CUBE_ROTATIONS, ElemId, HYPERCUBE_ROTATIONS};
use super::grips::{CUBE_GRIPS, GripId, HYPERCUBE_GRIPS};
use super::twists::Twist;

/// 3x3x3x3 facet-turning twisty puzzle.
#[static_init::dynamic]
pub static RUBIKS_4D: Puzzle = Puzzle::new(HYPERCUBE_GRIPS, &*HYPERCUBE_ROTATIONS);

/// 3x3x3 face-turning twisty puzzle.
#[static_init::dynamic]
pub static RUBIKS_3D: Puzzle = Puzzle::new(CUBE_GRIPS, &*CUBE_ROTATIONS);

pub struct Puzzle {
    /// List of grips. Each contains the list of twists that are available on
    /// that grip.
    pub grips: Vec<GripData>,
    /// Flattened list of twists. This contains the same information as
    /// [`Self::grips`], but flattened for easy enumeration and random sampling.
    pub twists: Vec<Twist>,
}
impl Puzzle {
    pub fn new(grips: impl IntoIterator<Item = GripId>, group: &[ElemId]) -> Self {
        let grips = grips
            .into_iter()
            .map(|id| GripData {
                id,
                transforms: id.transforms(group),
            })
            .collect_vec();

        let twists = grips
            .iter()
            .flat_map(|grip| {
                grip.transforms.iter().map(|&transform| Twist {
                    grip: grip.id,
                    transform,
                })
            })
            .collect_vec();

        Self { grips, twists }
    }

    pub fn random_moves(&self, count: usize) -> impl Iterator<Item = Twist> {
        let mut rng = rand::rng();
        (0..count).map(move |_| *self.twists.choose(&mut rng).unwrap())
    }
}

pub struct GripData {
    pub id: GripId,
    /// Elements from the grip group that fix this grip.
    pub transforms: Vec<ElemId>,
}
impl GripData {
    pub fn twists(&self) -> impl Iterator<Item = Twist> {
        let grip = self.id;
        self.transforms
            .iter()
            .map(move |&transform| Twist { grip, transform })
    }
}
