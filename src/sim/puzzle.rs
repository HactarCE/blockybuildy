use itertools::Itertools;
use rand::Rng;
use rand::seq::IndexedRandom;
use rayon::prelude::*;

use super::elements::{CUBE_ROTATIONS, ElemId, HYPERCUBE_ROTATIONS};
use super::grip_set::GripSet;
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

    pub fn grip_set(&self) -> GripSet {
        self.grips.iter().map(|g| g.id).collect()
    }

    pub fn random_moves(&self, rng: &mut impl Rng, count: usize) -> impl Iterator<Item = Twist> {
        (0..count).map(move |_| *self.twists.choose(rng).unwrap())
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
    pub fn par_twists(&self) -> impl ParallelIterator<Item = Twist> {
        let grip = self.id;
        self.transforms
            .par_iter()
            .map(move |&transform| Twist { grip, transform })
    }
}
