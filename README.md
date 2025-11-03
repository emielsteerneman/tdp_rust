# Me

So my problem is that I have tons of information. 2000 papers (named team description papers) from the RoboCup teams. This information entails so many topics. tons of research on path planning, AI, mechanics, control, engineering, physics, team management, etc etc etc. The problem that I have is that this information is quite sparse , possibly duplicated. e.g. one team might talk about some research they did on Rapidly exploring random tree, one team 3 years later has some other observation they kinda describe in their paper, one team from a different league describes some other related but different path planning method. When applying RAG I just get a lot of sparse information. What I want is dense information, but RAG doesn't seem suitable.. Ideally I would personally read through all 2000 papers and categorize and condense all information, and then list the sources of where I found this information. But of course that is humanly impossible.. what can I do?

# ChatGPT

You’re right: vanilla RAG is great at “recall,” terrible at **distilling** scattered, uneven content into something dense. The fix is to **separate recall from curation** and to pre-compute a structured, deduplicated “knowledge layer” you can cite. Here’s a concrete, buildable approach that works well for big, messy corpora like RoboCup TDPs.

## 1) Build a topic map first (unsupervised, then lock it)

1. **Section-aware chunking.** Split by headings (Introduction, Mechanics, Drivetrain, Path Planning, etc.), then into ~300–600-token passages. Keep rich metadata: `team, league, year, section, figures_present, equations_present`.
2. **Hybrid indexing.** BM25 + dense embeddings (e.g., bge/colbert). Add a **cross-encoder re-ranker** (msmarco-style) for top-k.
3. **Clustering → taxonomy.** For each major theme (Path Planning, Control, Drivetrain, Perception, Team Ops…), cluster embeddings; label clusters with LLM (map-reduce) to get a **stable topic taxonomy**:
   `topic > subtopic > aspect` (e.g., Path Planning > Sampling-based > RRT; Mechanics > Drivetrain > Omniwheels).

## 2) Extract “nuggets” (dense, citeable facts)

For every chunk, run an LLM extractor that outputs **atomic, source-grounded nuggets**:

```
Nugget {
  id, topic, subtopic, aspect,
  claim: one-sentence fact/insight,
  evidence_span: exact quoted span (≤25 words),
  strength: {empirical, anecdotal, design_choice, spec},
  context: {league, team, year, conditions},
  metrics: {numbers, units},
  citations: [doc_id#section#page]
}
```

Think “fact cards,” not summaries. This is crucial for de-dup and later synthesis.

## 3) Score “information density” and filter junk

Give each chunk and nugget an **InfoScore** to keep only dense material:

* **Novelty:** 1 – max cosine similarity vs. prior nuggets in same subtopic/time bin.
* **Specificity:** TF-IDF of technical terms + presence of numbers/units/equations.
* **Attribution:** has precise citation span? (binary boost)
* **Compression gain:** tokens_before / tokens_after_extraction (higher means denser).
* **Temporal recency:** soft boost for newer if not contradicting older.

`InfoScore = 0.35*Novelty + 0.25*Specificity + 0.2*Attribution + 0.1*Compression + 0.1*Recency`

Drop nuggets below a threshold (e.g., 0.55). You’ll eliminate intros/marketing fluff fast.

## 4) Semantic de-duplication (keep the best, keep the lineage)

Within each `(topic, subtopic, aspect, ~2-year window)`:

* **Textual de-dup:** MinHash/LSH on 5-word shingles to kill near copies.
* **Semantic merge:** cluster nuggets by embedding; **keep one canonical nugget** and **attach merged_sources[]** with differing teams/years and any contradictory edges.

Store:

```
CanonicalNugget {
  canonical_claim,
  merged_sources: [ {team, year, evidence_span, link} ... ],
  disagreements: [ids],
  metrics_distribution: {mean, sd, N},
}
```

## 5) Create “Topic Cards” (your dense layer)

For each (topic → subtopic), periodically auto-compile a **Topic Card**—a human-readable, *already condensed* page:

* **What it is** (2–3 lines)
* **Key design patterns / algorithms** with **canonical nuggets** (bullet list, each with 1–3 citations)
* **Typical parameters/metrics** (table aggregated from nuggets)
* **Known trade-offs & failure modes** (with citations)
* **Timeline highlights** (how thinking changed 2016→2025)
* **Open questions / gaps**

These cards are what your user sees at query time—not raw chunks.

## 6) Two-stage RAG at query time (“Recall → Distill”)

**Stage A (Recall):** retrieve 50–200 **nuggets** and **Topic Cards** (not raw text).
**Stage B (Distill):** small LLM composes an answer **only from nuggets/cards**, with inline citations.
Benefits: short contexts, high density, minimal hallucination.

## 7) Conflict handling & provenance

* If nuggets disagree, **surface both** with conditions (“RRT* improved success rate on cluttered fields at cost of CPU [A,B]; another team found no gain under X rules [C]”).
* Always provide **merged_sources** so users can drill down.

## 8) Practical filters that dramatically help

* **Team/league gates:** if the query is “omniwheels of RoboTeam Twente,” hard-filter to `team=RoboTeam Twente` first; fall back to `league=SSL` only if empty.
* **Section priors:** for hardware queries, upweight `Mechanics/Drivetrain/Wheels` sections; downweight `Introduction/Acknowledgements`.
* **Year windows:** prefer ±2–3 years unless user asks for history.
* **Figure/number priors:** chunks mentioning figures/tables/numbers get boosted.

## 9) Storage schema (minimal but effective)

* `chunks(indexed)`
* `nuggets (vector + keyword)`
* `canonical_nuggets (vector + relations)`
* `topic_cards (per subtopic)`
  Vector DB for similarity (FAISS/Qdrant/Weaviate), relational store for joins.

## 10) A tiny example: “RRT in SSL” (what the system produces)

**Canonical nugget**

* *Claim:* “RRT with dynamic obstacles: replan at 10–20 Hz; success ↑ ~15% vs. A* in midfield clutter; CPU 1.6×.”
* *Merged sources:* RoboTeam-A (2019 §Path Planning, Fig.3), RoboTeam-B (2022 §Planning), RoboTeam-C (2024 §Benchmarks).
* *Notes:* Benefits disappear with 3+ opponents moving >2 m/s (Team-B).

**Topic Card: Path Planning → Sampling-based → RRT**

* Pros/cons bullets, parameter ranges (step size, replan rate), known pitfalls (goal bias vs. jitter), and 5–8 citations.
  This is what your QA uses, not 40 disparate intros.

---

## Implementation checklist (you can start this week)

1. **Parser:** split PDFs/markdown; keep section headers + figure captions.
2. **Indexer:** BM25 + embeddings; store rich metadata.
3. **Extractor:** LLM prompt to emit Nuggets JSON; keep quoted evidence spans.
4. **Scoring:** compute InfoScore; persist; filter <0.55.
5. **De-dup:** MinHash + semantic cluster; build CanonicalNuggets.
6. **Card builder (cron):** per subtopic, summarize canonical nuggets into Topic Cards with citations.
7. **Query router:** map query → (topic/subtopic) → fetch cards+nuggets → compose answer.
8. **UI:** results first show **Topic Card** + expandable **sources**; raw paragraphs only on click.
9. **Eval:** measure precision@5 of citations, user time-to-insight, and % of “intro/ack” noise shown (aim <5%).
10. **Governance:** version your taxonomy; log merges; make it easy to roll back merges that were too aggressive.

---

## Why this beats plain RAG

* You’re not asking the model to synthesize from noisy 50-page contexts on the fly.
* You **front-load curation** into machine-generated, citeable nuggets and cards.
* You keep provenance tight and can **audit** every line back to a sentence in a paper.
* Over time, your Topic Cards become a living literature review of RoboCup.

If you want, share a small subset (say, 30 PDFs from 3 topics). I’ll sketch the Nugget extractor prompt, an InfoScore function, and a simple de-dup pipeline so you can test the idea end-to-end.


# Me

Let's start at step 1. So the problem that I have, is that I start with completely unlabeled data. I like clustering and taxonomy, but I don't even know what topics there are. I would like to throw all 2000 papers into ChatGPT in one go, but thats impossible. I have to go through the papers one by one, determine topics, and slowly somehow, by sequentially going through all papers over and over again, the topics should "settle". How do I do that? It kind of feels like a moving average that can only settle after a few rounds through all the papers

# ChatGPT

You don’t need to know the topics up front. Treat it as a **streaming / online topic discovery** problem: build seed topics from a small sample, then let topics “settle” as you pass through the corpus repeatedly with **damped, incremental updates** (and occasional split/merge). Here’s a concrete recipe that scales and converges.

## A. Prepare once (section-aware chunks + embeddings)

1. **Chunk smartly:** split by headings; within sections chunk to 300–600 tokens with 50–100 token overlap. Keep rich metadata: `team, league, year, section, has_figures, has_equations`.
2. **Normalize text:** lowercase, strip refs/URLs, keep numbers/units.
3. **Embeddings:** choose a cosine-friendly model (e.g., e5-large / bge-large). **L2-normalize** vectors so cosine ≡ dot product. Persist to a vector store (FAISS HNSW/IVF or Qdrant/Weaviate).

## B. Pass 0: discover seed topics on a subset

* Sample ~50k–100k chunks (or ~10–15% of all chunks).
* **Dim reduce** to 50–100 dims (PCA) to denoise, keep variance.
* **Find K and centroids cheaply:**

  * Run **spherical k-means** (MiniBatchKMeans on normalized vectors) for multiple K (e.g., 50, 100, 150).
  * Pick K with elbow on within-cluster cosine SSE **and** a minimum average silhouette (cosine) threshold (e.g., >0.15 for high-overlap corpora).
* Save: `topics = {topic_id -> centroid, members_sampled}`.

Optional: run **HDBSCAN** on the subset to propose natural dense groups; use those centroids to **initialize** k-means (best of both worlds).

## C. Streaming “moving-average” clustering over the full corpus

You now iterate over all chunks in **batches** and update topics with damping so they “settle.”

**Core idea (damped centroid update):**
For chunk embedding `x` assigned to topic `t` with centroid `c_t`,

```
c_t ← normalize( (1 − η) * c_t + η * x )
```

* `η` (learning rate) decays with iterations and cluster size, e.g. `η = η0 / sqrt(n_t)` with `η0 ≈ 0.5`.
* Assign to nearest centroid by **maximum cosine**. Use a rejection threshold (e.g., cos < 0.35 ⇒ call it **OUTLIER** for now).

**Do it with scikit-learn:**

* Use **MiniBatchKMeans** (cosine = Euclidean on normalized vectors) with `batch_size=2048…8192`.
* Run **2–3 epochs** over all chunks. After each epoch, compute diagnostics (below). Expect big moves in epoch 1, smaller in 2, tiny in 3.

**Speed tips**

* Pre-filter candidates: for each batch, retrieve top-N nearest centroids by dot product using an ANN index on centroids (N≈10) before exact max.
* Keep centroids in RAM; write periodic snapshots.

## D. Handle topic birth, splits, and merges (so it doesn’t “lock in” wrong)

Every N batches (e.g., 20k–50k items), run maintenance:

**1) Birth (new topics):**

* Collect **OUTLIER** assignments. If you see a pile of outliers with mutual cosine ≥ 0.6 and ≥ M members (e.g., M=200), promote to a **new topic** (centroid = mean). This catches genuinely new subfields.

**2) Merge (near duplicates):**

* If two centroids have cosine ≥ 0.92 and their member distributions across `team/section` heavily overlap, merge (weighted mean). Keep both IDs as **aliases** for provenance.

**3) Split (too broad):**

* If a topic’s intra-cluster average cosine drops below τ (e.g., 0.35), or it shows a **bimodal** centroid neighborhood, run a local 2–4-means **within that cluster** and accept the split if it increases average silhouette by ≥ 0.05.

## E. When to stop? (has it “settled”?)

Track three convergence curves per epoch:

1. **Centroid drift:** mean `1 − cos(c_t^{(k)}, c_t^{(k−1)})`. Stop when < 0.01 for 95% topics.
2. **Assignment stability:** % of points that switch topics between epochs. Stop when < 8% then < 3%.
3. **Topic churn:** (# births + merges + splits) / total topics. Stop when < 2% per epoch.

If you like hard numbers: 2–3 full passes usually suffice on technical corpora.

## F. Name and describe topics automatically (and stably)

For each topic after each epoch:

* **Keyphrases:** take top 200 members by cosine to centroid, extract n-grams/TF-IDF + KP extraction (e.g., YAKE/RKEA).
* **Auto-name:** prompt an LLM with the top keyphrases, dominant sections, and 3–5 representative snippets to produce a **≤6-word title** and a **one-liner**.
* **Stability:** keep **topic_id** permanent; only update `display_name` if Jaccard overlap of top keyphrases with previous epoch > 0.6 (prevents name thrash).

Store **topic facets** (distribution over `section`, `year`, `league`)—they’re great priors at query time.

## G. Build a light taxonomy from the clusters

Turn flat topics into a 2-level tree:

* Compute a topic–topic similarity graph (centroid cosine).
* Run **Leiden**/Louvain to detect **super-topics** (e.g., “Path Planning”).
* Topic display path: `SuperTopic → Topic`.
* You can re-run this cheaply after each epoch; it stabilizes quickly.

## H. Quality gates so your topics are *useful*, not just pretty

For each topic, compute:

* **Coherence** (c_npmi) on the top keyphrases. Drop/merge topics with low coherence.
* **Density:** mean cosine of members to centroid.
* **Specificity:** section entropy (low entropy = focused sections like “Mechanics/Drivetrain”).
* **Recency skew:** check that you didn’t overfit to new papers only.

Use these to flag suspect topics for split/merge or demotion.

## I. Make it pragmatic (what to implement first)

Minimal code sketch (pseudo):

```python
from sentence_transformers import SentenceTransformer
from sklearn.cluster import MiniBatchKMeans
import numpy as np

# 1) load chunk embeddings (normalized)
X = load_memmap_vectors()           # shape (N, d), L2-normalized
mb = MiniBatchKMeans(
    n_clusters=120, batch_size=4096, max_iter=1, n_init='auto', init='k-means++'
)

# 2) Pass 0: fit on a sample to get seeds
idx = np.random.choice(len(X), size=100_000, replace=False)
mb.partial_fit(X[idx])

# 3) Epoch loop over all data
for epoch in range(3):
    for batch in iterate_batches(X, 4096):
        mb.partial_fit(batch)
    # diagnostics
    labels = mb.predict(X_sample_for_eval)
    eval_and_persist(mb.cluster_centers_, labels)
    # maintenance: birth/merge/split on centroids & recent assignments
```

Add:

* OUTLIER buffer via assignment cosine threshold.
* Birth/split/merge maintenance between epochs.
* Topic naming using top-members per centroid.

## J. What you do with the topics next

* Use topics to **route RAG** (filter by topic first, then retrieve), and to **build Topic Cards** later.
* Log per-topic **representative chunks** (top-k by centroid cosine) for quick inspection.
* Persist **topic versions** (v1, v2, …) so you can diff and roll back.

## K. Practical knobs (defaults that work)

* Chunk size 300–600 tokens, overlap 80.
* Embedding: e5-large / bge-large (cosine). Normalize always.
* Seed K: 100–150 for 2000 papers (~150k–300k chunks).
* Cosine thresholds: outlier < 0.35, merge > 0.92, split if density < 0.35.
* Epochs: 2–3. Batch size 2k–8k.
* Dim reduce to 64 for clustering speed.

## L. Common failure modes & fixes

* **Intro/ack clusters**: high entropy topics of fluff. Fix: downweight those sections or pre-filter by `section ∉ {Introduction, Acknowledgements}` for clustering; keep them but as a single “Meta” topic.
* **Year/league leaks**: some topics cluster by team/year prose style. Fix: whiten embeddings by regressing out `team/year` one-hot projections, or use adversarial removal of those features before clustering.
* **Topic drift**: if you ingest many new papers, keep a **frozen centroid copy** and limit η for mature topics; allow **birth** more readily than moving old centroids.