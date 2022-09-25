use crypto_hash::{digest, Algorithm};
use rand::prelude::*;
use rand_chacha::*;

pub type VertexIndex = usize;

pub type Secret = [u8];

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub enum Color {
    Red,
    Green,
    Blue,
}

impl Color {
    pub fn matches(&self, commitment: ZKCommitment, secret: &Secret) -> bool {
        digest(Algorithm::SHA256, &[&[(*self as u8)], secret].concat()) == commitment.hash
    }
}

impl Commitable for Color {
    fn commit(&self) -> Commitment {
        let mut rng = ChaCha20Rng::from_entropy();

        let mut secret: [u8; 32] = [0; 32];
        rng.fill_bytes(&mut secret);

        Commitment {
            secret,
            hash: digest(
                Algorithm::SHA256,
                &[&[(*self as u8)], &secret as &[u8]].concat(),
            ),
        }
    }
}

#[derive(Clone)]
pub struct Vertex<T: Clone> {
    contents: Option<T>,
    adjacent: Vec<VertexIndex>,
}

impl<T: Clone + Commitable> Vertex<T> {
    fn commit(&self) -> Vertex<Commitment> {
        Vertex {
            contents: Some(self.contents.as_ref().unwrap().commit()),
            adjacent: self.adjacent.clone(),
        }
    }
}

impl Vertex<Commitment> {
    fn zk_commit(&self) -> Vertex<ZKCommitment> {
        Vertex {
            contents: Some(self.contents.as_ref().unwrap().zk_commit()),
            adjacent: self.adjacent.clone(),
        }
    }
}

pub struct Graph<T: Clone> {
    vertices: Vec<Vertex<T>>,
}

impl<T: Clone + Commitable> Graph<T> {
    pub fn new() -> Graph<T> {
        Graph { vertices: vec![] }
    }

    fn clone(&self) -> Graph<T> {
        Graph {
            vertices: self.vertices.clone(),
        }
    }

    pub fn color(&mut self, vertex: VertexIndex, color: T) {
        self.vertices[vertex].contents = Some(color);
    }

    pub fn add_vertex(&mut self) -> VertexIndex {
        self.vertices.push(Vertex {
            adjacent: vec![],
            contents: None,
        });

        self.vertices.len() - 1
    }

    pub fn make_adjacent(&mut self, a: VertexIndex, b: VertexIndex) {
        if a < self.vertices.len() && b < self.vertices.len() {
            if a != b {
                self.vertices[a].adjacent.push(b);
                self.vertices[b].adjacent.push(a);
            }
        }
    }

    pub fn colors_for(&self, a: VertexIndex, b: VertexIndex) -> Option<(T, T)> {
        if a < self.vertices.len() && b < self.vertices.len() {
            Some((
                self.vertices[a].contents.clone().unwrap(),
                self.vertices[b].contents.clone().unwrap(),
            ))
        } else {
            None
        }
    }

    pub fn commit(&self) -> Graph<Commitment> {
        let mut commitment: Graph<Commitment> = Graph::<Commitment>::new();

        for vertex in &self.vertices {
            commitment.vertices.push(vertex.commit());
        }

        commitment
    }

    pub fn len(&self) -> usize {
        self.vertices.len()
    }

    pub fn get_adjacent(&self, v: VertexIndex) -> Vec<VertexIndex> {
        if v < self.vertices.len() {
            self.vertices[v].adjacent.clone()
        } else {
            vec![]
        }
    }
}

impl Graph<Color> {
    pub fn random_permutation(&self) -> Graph<Color> {
        let mut random_g: Graph<Color> = self.clone();
        let mapping: ColorToColorMap = ColorToColorMap::new_random();

        for mut vertex in &mut random_g.vertices {
            vertex.contents = mapping.from(&vertex.contents);
        }

        random_g
    }
}

impl Graph<Commitment> {
    pub fn zk_commit(&self) -> Graph<ZKCommitment> {
        let mut zk_commitment: Graph<ZKCommitment> = Graph::<ZKCommitment>::new();

        for vertex in &self.vertices {
            zk_commitment.vertices.push(vertex.zk_commit());
        }

        zk_commitment
    }
}

struct ColorToColorMap {
    colors: [Color; 3],
}

impl ColorToColorMap {
    fn new_random() -> ColorToColorMap {
        let mut random_map = ColorToColorMap {
            colors: [Color::Red, Color::Green, Color::Blue],
        };

        // Randomly choose a coloring
        let mut rng = ChaCha20Rng::from_entropy();
        random_map.colors.shuffle(&mut rng);

        random_map
    }

    fn from(&self, optional_color: &Option<Color>) -> Option<Color> {
        optional_color
            .as_ref()
            .map(|color| self.colors[*color as usize].clone())
    }
}

#[derive(Clone)]
pub struct ZKCommitment {
    hash: Vec<u8>,
}

#[derive(Clone)]
pub struct Commitment {
    pub secret: [u8; 32],
    hash: Vec<u8>,
}

pub trait Commitable {
    fn commit(&self) -> Commitment;
}

impl Commitment {
    fn zk_commit(&self) -> ZKCommitment {
        ZKCommitment {
            hash: self.hash.clone(),
        }
    }
}

impl Commitable for Commitment {
    fn commit(&self) -> Commitment {
        self.clone()
    }
}

impl Commitable for ZKCommitment {
    fn commit(&self) -> Commitment {
        Commitment {
            secret: [0; 32],
            hash: self.hash.clone(),
        }
    }
}
