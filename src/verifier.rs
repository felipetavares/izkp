use crate::graph::{Color, Graph, Secret, VertexIndex, ZKCommitment};
use rand::prelude::*;
use rand_chacha::*;
use std::fmt;

const ACCEPTANCE_CONFIDENCE: f64 = 0.9999;

pub enum Result {
    Undecided,
    Accepted,
    Rejected,
}

impl fmt::Display for Result {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Result::Undecided => write!(f, "undecided"),
            Result::Accepted => write!(f, "accepted"),
            Result::Rejected => write!(f, "rejected"),
        }
    }
}

pub struct Verifier {
    graph_size: f64,

    last_requested_vertices: Option<(VertexIndex, VertexIndex)>,
    last_commitment: Option<Graph<ZKCommitment>>,

    pub confidence: f64,
    pub result: Result,
}

impl Verifier {
    pub fn new() -> Verifier {
        Verifier {
            confidence: 0.0,
            result: Result::Undecided,
            graph_size: 0.0,
            last_requested_vertices: None,
            last_commitment: None,
        }
    }

    pub fn choose_random_vertices(
        &mut self,
        commitment: Graph<ZKCommitment>,
    ) -> Option<(VertexIndex, VertexIndex)> {
        if !matches!(self.result, Result::Undecided) {
            return None;
        }

        let mut rng = ChaCha12Rng::from_entropy();
        let vertex: VertexIndex = rng.gen_range(0..commitment.len());
        let adjacent = commitment.get_adjacent(vertex);

        self.graph_size = commitment.len() as f64;
        self.last_commitment = Some(commitment);

        if adjacent.len() > 0 {
            self.last_requested_vertices = Some((vertex, adjacent[0]));

            println!(
                "Verifier asked for vertices {:?}",
                self.last_requested_vertices
            );

            self.last_requested_vertices
        } else {
            None
        }
    }

    pub fn verify_coloring(
        &mut self,
        color_a: Color,
        salt_a: &Secret,
        color_b: Color,
        salt_b: &Secret,
    ) {
        if !matches!(self.result, Result::Undecided) {
            return;
        }

        println!("Prover shared colors ({:?}, {:?})", color_a, color_b);

        // Adjacent colors in the graph cannot be equal!
        if color_a == color_b {
            self.reject();
            return;
        } else {
            let (vertex_a, vertex_b) = self.last_requested_vertices.unwrap();
            let (commitment_a, commitment_b) = self
                .last_commitment
                .as_ref()
                .unwrap()
                .colors_for(vertex_a, vertex_b)
                .unwrap();

            // Revealed colors must match the pre-commitment
            if color_a.matches(commitment_a, salt_a) && color_b.matches(commitment_b, salt_b) {
                println!("Shared colors match the hashed & salted pre-commitment!");

                self.increase_confidence();
            } else {
                self.reject();
                return;
            }
        }

        if self.confidence > ACCEPTANCE_CONFIDENCE {
            self.accept();
        }
    }

    fn reject(&mut self) {
        self.result = Result::Rejected;
        self.confidence = 1.0;
    }

    fn accept(&mut self) {
        self.result = Result::Accepted;
    }

    fn increase_confidence(&mut self) {
        self.confidence += (1.0 - self.confidence) / self.graph_size;
    }
}
