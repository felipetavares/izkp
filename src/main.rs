mod graph;
mod verifier;

use graph::*;
use verifier::*;

// Returns a 3-colorable graph equivalent to the statement x + 1 = 2 given
// True = 1, False = 0.
fn build_graph() -> Graph<Color> {
    let mut g = Graph::<Color>::new();

    // Add all vertices to the graph ahead of time
    let t: VertexIndex = g.add_vertex();
    let f: VertexIndex = g.add_vertex();
    let n: VertexIndex = g.add_vertex();
    let x: VertexIndex = g.add_vertex();

    // Construct the palette
    g.make_adjacent(t, f);
    g.make_adjacent(f, n);
    g.make_adjacent(n, t);

    g.color(t, Color::Green);
    g.color(f, Color::Red);
    g.color(n, Color::Blue);

    // Add in x so that it must be set to true when coloring the graph
    g.make_adjacent(f, x);
    g.make_adjacent(n, x);

    g.color(x, Color::Green);

    g
}

fn prover() {
    let statement: Graph<Color> = build_graph();
    let mut verifier = Verifier::new();

    while matches!(verifier.result, Result::Undecided) {
        // Create a random coloring of the statement and commit to it using
        // hashes of the coloring.
        let random_graph: Graph<Color> = statement.random_permutation();
        let commitment: Graph<Commitment> = random_graph.commit();

        // Remove all information but the hashes from the commitment and send to
        // the verifier, which asks the colors of two vertices.
        let (vertex_a, vertex_b) = verifier
            .choose_random_vertices(commitment.zk_commit())
            .unwrap();

        // Check the colors and salt we used for those vertices...
        let (color_a, color_b) = random_graph.colors_for(vertex_a, vertex_b).unwrap();
        let (salt_a, salt_b) = commitment.colors_for(vertex_a, vertex_b).unwrap();

        // ...and send them for the verifier to check
        verifier.verify_coloring(color_a, &salt_a.secret, color_b, &salt_b.secret);
    }

    // Now the verifier is not `Undecided` anymore, so we show the decision.
    println!(
        "The verifier deems the proof {} and is {:.4}% convinced.",
        verifier.result,
        verifier.confidence * 100.0
    );
}

fn main() {
    prover();
}
