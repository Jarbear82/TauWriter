Here's a unified, corrected, and reorganized graph theory reference that merges your original taxonomy with the gaps identified across all four reviews — organized by *conceptual category* (types vs. properties vs. operations vs. algorithms) rather than mixing them together, since that mixing was the most common critique.

## 1. Foundational Vocabulary

- **Graph**: $G = (V, E)$ — a set of vertices and a set of edges connecting them.
- **Order**: $|V|$ (vertex count). **Size**: $|E|$ (edge count).
- **Degree** $\deg(v)$: number of edges incident to a vertex. Directed graphs split this into **in-degree** and **out-degree**.
- **Degree Sequence**: the sorted list of all vertex degrees. The **Handshaking Lemma** states the sum of all degrees equals $2|E|$. A **Graphical Sequence** is one that Erdős–Gallai conditions confirm can actually be realized as a graph.
- **Neighborhood**: the set of vertices adjacent to $v$ (open = excludes $v$; closed = includes $v$).
- **Adjacency** (two vertices linked) vs. **Incidence** (a vertex is an endpoint of an edge).

### Walks, Trails, Paths, Cycles
These are often conflated but are formally distinct, and the distinction underlies Eulerian/Hamiltonian definitions and BFS/DFS semantics:
- **Walk**: vertices/edges may repeat.
- **Trail**: no repeated edges (vertices may repeat).
- **Path**: no repeated vertices.
- **Circuit**: a closed trail (starts/ends at same vertex).
- **Cycle**: a closed path.

### Distance Metrics
- **Distance**: shortest-path length between two vertices.
- **Eccentricity**: max distance from a vertex to any other.
- **Radius**: min eccentricity in the graph. **Diameter**: max eccentricity.
- **Center**: vertices with eccentricity = radius. **Periphery**: vertices with eccentricity = diameter.
- **Girth**: length of the shortest cycle. **Circumference**: length of the longest cycle.

## 2. Structural Models

- **Flat Graph**: all nodes/edges on one plane (no nesting).
- **Compound / Clustered / Hierarchical Graph**: nodes can contain sub-graphs recursively. More common in visualization and graph databases than in pure math theory.
- **Standard Graph**: edges are binary. Note the precise distinction: a **directed edge** is an ordered pair $(u,v) \in V \times V$; an **undirected edge** is an unordered pair $\{u,v\} \in \binom{V}{2}$ — "ordered pair" only strictly applies to the directed case.
- **Hypergraph**: edges (hyperedges) connect $N$ vertices at once. Reifiable into a standard bipartite **incidence graph** (one vertex-set for originals, one for hyperedges). Variants: **Directed Hypergraph** (hyperedges have source/target sets), **Weighted/Signed Hypergraph**.
- **Simplicial Complex / Cell Complex**: higher-order generalizations beyond hypergraphs, closed under taking faces.

### Memory Representations *(implementation, not a graph-theoretic type)*
- **Edge List**: cache-friendly array of pairs; good for full-edge iteration.
- **Adjacency List**: per-vertex neighbor lists; standard for sparse graphs.
- **Adjacency Matrix**: $O(V^2)$ grid; best for dense graphs.
- **Incidence Matrix**: vertices × edges; $O(V \times E)$ — typically *larger* than an adjacency matrix, so it's not "for dense graphs" generally — it's most useful for hypergraph encoding and cycle-space linear algebra.
- Also worth knowing: **Compressed Sparse Row/Column (CSR/CSC)**, hashed adjacency maps, forward/reverse star representations (used in large-scale/GPU graph engines).

## 3. Classification Dimensions

### Edge Attributes
- **Directed / Undirected / Mixed** (contains both).
- **Weighted / Unweighted**.
- **Signed Graph**: edges carry $+/-$ signs (trust networks, balance theory).
- **Labeled / Attributed / Property Graph**: distinct concepts — labels are simple tags, attributes are arbitrary properties, and a **Property Graph** formally means both nodes and edges carry key-value data (the model behind Neo4j-style graph databases).

### Structural Constraints
- **Simple**: no loops, no parallel edges.
- **Multigraph**: parallel edges allowed. (Definitions vary on whether loops are permitted — be explicit: *loopless multigraph* vs. *multigraph with loops*.)
- **Pseudograph**: loops and parallel edges both allowed.

### Completeness & Structure
- **Null / Empty Graph**: most standard definition is zero vertices, zero edges. A graph with vertices but no edges is more precisely an **Edgeless Graph**. **Trivial Graph**: exactly one vertex, no edges. (Terminology varies across sources — worth stating your convention explicitly.)
- **Complete** ($K_n$), **Regular**, **Bipartite**, **Complete Bipartite** ($K_{m,n}$), **Complete Multipartite**, **Turán Graph** (multipartite with maximally balanced parts).
- **Cyclic / Acyclic**.
- **Sparse / Dense**: these are density classifications, generally asymptotic ($|E| = O(|V|)$ vs. $O(|V|^2)$), not strict binary types.

## 4. Connectivity & Robustness

- **Connected / Disconnected**, **Strongly / Weakly Connected** (directed).
- **Connected Component**, **Strongly Connected Component (SCC)**, **Weakly Connected Component**, **Biconnected Component** (maximal subgraph with no articulation points).
- **Bridge (Cut-Edge)**: removal disconnects the graph.
- **Articulation Point (Cut-Vertex)**: removal disconnects the graph.
- **Vertex Connectivity ($\kappa$)** / **Edge Connectivity ($\lambda$)**: minimum removals needed to disconnect the graph — a numeric measure of robustness.
- **$k$-Connected / $k$-Edge-Connected**.
- **Menger's Theorem**: links min cut size to max number of disjoint paths between two vertices.
- **Cut / $s$–$t$ Cut / Minimum Cut**: partitions of vertices, central to flow problems and clustering.
- **Condensation Graph**: the DAG formed by contracting each SCC of a directed graph into one node.

## 5. Substructures: Cliques, Matchings, Covers

- **Subgraph** / **Induced Subgraph** / **Spanning Tree** (spanning **Forest** if disconnected).
- **Clique** (all pairs adjacent) / **Independent Set** (no pairs adjacent).
- **Matching**: edges sharing no vertices. **Maximum** vs. **Maximal** vs. **Perfect Matching**.
- **Vertex Cover** / **Edge Cover** / **Dominating Set**.
- **König's Theorem**: in bipartite graphs, max matching = min vertex cover. **Hall's Marriage Theorem**: condition for a perfect matching to exist in bipartite graphs.

## 6. Graph Operations & Transformations

- **Complement Graph**: edges exist exactly where the original graph has none.
- **Line Graph**: original edges become new vertices; adjacency = shared endpoint.
- **Union / Intersection / Disjoint Union / Join**.
- **Graph Products**: Cartesian, Tensor, Strong, Lexicographic (e.g., Cartesian product of two paths = a grid graph).
- **Vertex/Edge Deletion**, **Edge Contraction**, **Subdivision** (replace edge with a path), **Graph Power** (connect vertices within distance $k$).
- **Dual Graph**: for planar graphs — faces become vertices, adjacent faces become edges.
- **Transpose/Reverse Graph** (directed): reverse every edge direction.
- **Graph Minor**: obtained via deletion + contraction. Note: minors are the formal basis for collapsing structure via contraction generally, but they are not literally "the foundation" of compound-graph node collapsing — the two are related, not equivalent.

## 7. Trees (expanded)

- **Tree**: connected, acyclic, undirected. **Forest**: disjoint union of trees.
- **Rooted Tree**: one vertex designated root — introduces parent/child/ancestor/descendant/depth/height/leaf/internal node.
- **Binary Tree** and variants (full, complete, perfect, balanced, degenerate); **$k$-ary Tree**; **Ordered Tree** (child order matters).
- **Arborescence**: directed version of a rooted tree (in-arborescence vs. out-arborescence). **Polytree**: a DAG whose underlying undirected graph is a tree.

## 8. Named Graph Families

Star, Path Graph ($P_n$), Cycle Graph ($C_n$), Wheel, Fan, Grid/Lattice, Hypercube, Cactus (cycles share ≤1 vertex), Circulant, Cayley (built from a group + generators), Block Graph, Series-Parallel Graph.

**Structural classes**: Chordal (every cycle ≥4 has a chord), Interval, Perfect (chromatic number = clique number for every induced subgraph), Comparability, Cograph, Split, Threshold, Outerplanar, Expander/Ramanujan Graph.

**Directed/other**: DAG, Tournament, Eulerian Graph (edge-visiting circuit), Hamiltonian Graph (vertex-visiting cycle), Pseudoforest/Functional Graph (each vertex has exactly one outgoing edge — used in cycle detection).

## 9. Planarity & Topology

- **Planar**: drawable with no edge crossings. **Plane Graph**: a specific crossing-free embedding (embedding is a property of a *drawing*, planarity is a property of the *graph*).
- **Crossing Number**: minimum crossings required.
- **Kuratowski's Theorem**: planar ⟺ no subdivision of $K_5$ or $K_{3,3}$. **Wagner's Theorem**: the minor-based equivalent.
- **Euler's Formula**: $|V| - |E| + |F| = 2$ for connected planar graphs.
- **Genus**: minimum handles needed for a crossing-free embedding on a surface. **Book Thickness / Thickness**.
- **Graph Minors Theorem (Robertson–Seymour)**: any minor-closed graph family has a finite forbidden-minor characterization.
- **Geometric/Spatial Graphs**: Unit Disk Graph, Visibility Graph, Delaunay Triangulation, Gabriel Graph, Relative Neighborhood Graph, Intersection Graph — relevant to layout, rendering, and collision detection.

## 10. Coloring

Proper Vertex Coloring, Edge Coloring (Vizing's Theorem), Chromatic Number, Chromatic Index, Chromatic Polynomial, List Coloring, Clique Number, Independence Number.

## 11. Symmetry & Equivalence

- **Isomorphism**: structural equivalence between two graphs. **Automorphism**: isomorphism from a graph to itself (internal symmetry). **Subgraph Isomorphism**: search problem for whether one graph contains another. Note: graph isomorphism is GI-complete — not known to be in P or NP-complete.

## 12. Matrix & Spectral Graph Theory

- **Degree Matrix**, **Adjacency Matrix**, **Laplacian** ($L = D - A$), **Signless Laplacian** ($Q = D+A$), **Normalized/Random-Walk Laplacian**.
- **Distance Matrix**, **Incidence Matrix**.
- **Spectrum**: eigenvalues of the adjacency or Laplacian matrix. **Spectral Radius**.
- **Algebraic Connectivity** (Fiedler value — second-smallest Laplacian eigenvalue) and its eigenvector, the **Fiedler Vector**, used for spectral partitioning/clustering.
- **Cheeger Inequality**: links spectral gap to graph expansion.

## 13. Structural Width Parameters

**Treewidth**, **Pathwidth**, **Clique-width**, **Branchwidth**, **Degeneracy** — central to modern parameterized/fixed-parameter-tractable algorithms.

## 14. Random & Generative Models

- **Erdős–Rényi $G(n,p)$**: each edge independently present with probability $p$ — the network-science baseline model.
- **Configuration Model**: random graph matching a target degree sequence.
- **Stochastic Block Model**: generates community structure.
- **Watts–Strogatz**: rewired lattice → small-world behavior.
- **Barabási–Albert**: preferential attachment → scale-free behavior.
- **Random Geometric Graph**: edges from spatial proximity.

## 15. Network Science & Applied Metrics

- **Scale-Free** (power-law degree distribution, hub-dominated) and **Small-World** (short paths + high clustering) graphs.
- **Centrality**: Degree, Betweenness, Closeness, Eigenvector, PageRank, Katz, Harmonic.
- **Clustering Coefficient**, **Assortativity**, **Reciprocity**, **Modularity**, **Community Structure**, **Motifs/Graphlets**, **Rich-Club Coefficient**, **Core-Periphery Structure**.

## 16. Dynamic, Multilayer & Uncertain Graphs

- **Temporal/Dynamic Graph**: edges/vertices change or carry timestamps.
- **Multilayer / Multiplex Graph**: multiple relationship types over the same or overlapping vertex sets.
- **Heterogeneous Graph**: multiple node/edge types (common in knowledge graphs).
- **Probabilistic / Fuzzy Graph**: edges exist with probabilities or membership degrees.
- **Bayesian Network** (directed, conditional-dependency DAG) vs. **Markov Network** (undirected).

## 17. Computational Graph Models

**Property Graph** (Neo4j-style), **RDF Graph** (subject–predicate–object triples), **Knowledge Graph**, **Ontology Graph** — increasingly the model relevant to graph databases and query languages (Cypher, Gremlin, SPARQL, GQL).

## 18. Core Algorithms *(distinct from graph types — worth its own section)*

- **Traversal**: BFS, DFS, Topological Sort, Reachability/Transitive Closure.
- **Shortest Paths**: Dijkstra, Bellman-Ford, Floyd-Warshall, A*.
- **Spanning Structures**: Minimum Spanning Tree (Kruskal, Prim).
- **Components**: Tarjan's/Kosaraju's algorithms for SCCs.
- **Matching**: Hungarian Algorithm.
- **Flow**: Ford-Fulkerson, Edmonds-Karp, Dinic (Max-Flow Min-Cut Theorem).
- **Isomorphism testing**: nauty, bliss, VF2 (practical) vs. GI-completeness (theoretical).

## Practical Use-Case Mapping

| Use Case | Relevant Concepts |
|---|---|
| Build systems / dependency resolution | DAG, topological sort |
| Version control, Git | DAG, arborescence |
| Social networks | Scale-free, small-world, centrality, community detection |
| Routing / GPS | Weighted shortest paths, A* |
| Compiler/network scheduling | Interval graphs, chordal graphs |
| Graph databases | Property graphs, RDF, labeled multigraphs |
| Rendering/collision detection | Intersection graphs, geometric graphs |
| Sparse matrix factorization | Chordal graphs, treewidth |
| ML on graphs | GNNs, spectral clustering, Weisfeiler-Leman test |
| Network robustness/infrastructure | $k$-connectivity, bridges, articulation points |

---

**Key corrections to the original doc** worth flagging: the "ordered pair" edge definition applies strictly to directed edges, not undirected ones; Null/Trivial/Edgeless graph terminology needs an explicit convention since sources disagree; Sparse/Dense are density classifications rather than strict types; and clique/independent set/chromatic number are graph *invariants*, not graph *types*, so they sit more naturally in a properties section than a taxonomy of graph forms.
