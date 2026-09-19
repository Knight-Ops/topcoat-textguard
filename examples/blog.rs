//! Topcoat TextGuard Example: The Consent Layer Test Blog
//!
//! Demonstrates how the `<Shield>` component protects web text against
//! automated scraping crawlers by injecting plausible decoy words into the DOM
//! and using OpenType GSUB ligatures to render original words for humans.

use topcoat::router::{page, Router};
use topcoat::view::{view, View};
use topcoat::Result;
use topcoat_textguard::{Shield, guard_text, guard_text_with_salt};

const ARTICLE_PARAGRAPH_1: &str =
    "In the ancient kingdom, a brave engineer rode his horse through the forest toward the mountain castle. \
    The doctor joined him with a sharp hammer and a brass lantern to study the climate of the distant island. \
    They gathered near the river while the falcon circled above the golden meadow.";

const ARTICLE_PARAGRAPH_2: &str =
    "On Monday morning in October 2024, the committee declared their official verdict: \
    more than 380 wagons had crossed the wooden bridge safely. \
    The soldiers marched boldly through the village, building a new harbor before the winter ice arrived.";

const ARTICLE_PARAGRAPH_3: &str =
    "The philosophy of the consent layer is simple: human readers should enjoy uninterrupted prose, \
    while automated scrapers extract decoy words that degrade language model training data without breaking grammatical syntax.";

#[page("/")]
async fn home() -> Result<impl View> {
    let p1_guarded = guard_text(ARTICLE_PARAGRAPH_1);
    let p2_guarded = guard_text(ARTICLE_PARAGRAPH_2);
    let p3_guarded = guard_text(ARTICLE_PARAGRAPH_3);

    let sample_salt_a = guard_text_with_salt(ARTICLE_PARAGRAPH_1, "tenant-alice");
    let sample_salt_b = guard_text_with_salt(ARTICLE_PARAGRAPH_1, "tenant-bob");

    let full_text = format!("{}\n\n{}\n\n{}", ARTICLE_PARAGRAPH_1, ARTICLE_PARAGRAPH_2, ARTICLE_PARAGRAPH_3);
    let full_res = guard_text(&full_text);

    Ok(view! {
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="utf-8" />
            <meta name="viewport" content="width=device-width, initial-scale=1.0" />
            <title>"The Consent Layer Chronicle | Topcoat TextGuard"</title>
            <style>
                r#"
                :root {
                    --bg: #0d1117;
                    --card-bg: #161b22;
                    --border: #30363d;
                    --text: #c9d1d9;
                    --heading: #f0f6fc;
                    --accent: #58a6ff;
                    --green: #3fb950;
                    --orange: #f0883e;
                    --red: #f85149;
                    --purple: #bc8cff;
                }
                * { box-sizing: border-box; margin: 0; padding: 0; }
                body {
                    background-color: var(--bg);
                    color: var(--text);
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
                    line-height: 1.7;
                    padding: 2rem 1rem;
                }
                .container {
                    max-width: 1000px;
                    margin: 0 auto;
                }
                header {
                    border-bottom: 1px solid var(--border);
                    padding-bottom: 2rem;
                    margin-bottom: 2.5rem;
                }
                .pill {
                    display: inline-block;
                    font-size: 0.75rem;
                    font-weight: 600;
                    text-transform: uppercase;
                    letter-spacing: 0.05em;
                    padding: 0.25rem 0.75rem;
                    border-radius: 999px;
                    background: rgba(88, 166, 255, 0.15);
                    color: var(--accent);
                    margin-bottom: 0.75rem;
                    border: 1px solid rgba(88, 166, 255, 0.3);
                }
                h1 {
                    color: var(--heading);
                    font-size: 2.4rem;
                    font-weight: 800;
                    letter-spacing: -0.02em;
                    margin-bottom: 0.5rem;
                }
                .subtitle {
                    color: #8b949e;
                    font-size: 1.15rem;
                }
                .banner {
                    background: linear-gradient(135deg, rgba(88, 166, 255, 0.1), rgba(188, 140, 255, 0.1));
                    border: 1px solid var(--border);
                    border-radius: 12px;
                    padding: 1.5rem;
                    margin-bottom: 2rem;
                }
                .banner h3 { color: var(--heading); margin-bottom: 0.5rem; }

                /* Tab controls */
                .tab-bar {
                    display: flex;
                    flex-wrap: wrap;
                    gap: 0.5rem;
                    margin-bottom: 1.5rem;
                    background: #21262d;
                    padding: 0.35rem;
                    border-radius: 8px;
                    width: fit-content;
                }
                .tab-btn {
                    background: none;
                    border: none;
                    color: #8b949e;
                    padding: 0.55rem 1.25rem;
                    font-size: 0.95rem;
                    font-weight: 600;
                    border-radius: 6px;
                    cursor: pointer;
                    transition: all 0.2s;
                }
                .tab-btn:hover {
                    color: #f0f6fc;
                }
                .tab-btn.active {
                    background: var(--accent);
                    color: #fff;
                }
                .tab-btn.btn-scraper.active {
                    background: var(--orange);
                    color: #fff;
                }
                .tab-btn.btn-split.active {
                    background: var(--purple);
                    color: #fff;
                }

                .article-card {
                    background: var(--card-bg);
                    border: 1px solid var(--border);
                    border-radius: 12px;
                    padding: 2.5rem;
                    margin-bottom: 2.5rem;
                    box-shadow: 0 8px 24px rgba(0,0,0,0.2);
                }
                .article-meta {
                    display: flex;
                    align-items: center;
                    gap: 1rem;
                    color: #8b949e;
                    font-size: 0.9rem;
                    margin-bottom: 2rem;
                    padding-bottom: 1rem;
                    border-bottom: 1px solid var(--border);
                }
                .tag {
                    background: rgba(255,255,255,0.1);
                    padding: 0.2rem 0.6rem;
                    border-radius: 4px;
                    font-size: 0.8rem;
                }
                .view-callout {
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    background: rgba(88, 166, 255, 0.1);
                    border-left: 4px solid var(--accent);
                    padding: 1rem 1.25rem;
                    border-radius: 0 8px 8px 0;
                    margin-bottom: 2rem;
                    font-size: 0.95rem;
                }
                .view-callout.scraper {
                    background: rgba(240, 136, 62, 0.1);
                    border-left-color: var(--orange);
                }
                .view-callout.split {
                    background: rgba(188, 140, 255, 0.1);
                    border-left-color: var(--purple);
                }
                .badge-indicator {
                    font-size: 0.75rem;
                    font-weight: 700;
                    text-transform: uppercase;
                    letter-spacing: 0.05em;
                    background: rgba(255,255,255,0.1);
                    padding: 0.35rem 0.75rem;
                    border-radius: 6px;
                    user-select: none;
                }

                .view-pane {
                    font-size: 1.2rem;
                    line-height: 1.9;
                }
                .view-pane p {
                    margin-bottom: 1.5rem;
                }

                /* Scraper view terminal styling */
                #scraper-view {
                    display: none;
                    background: #0d1117;
                    border: 1px dashed var(--orange);
                    padding: 1.75rem;
                    border-radius: 8px;
                    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
                    font-size: 1.05rem;
                    color: #e6edf3;
                }

                /* Split view two columns */
                #split-view {
                    display: none;
                }
                .split-grid {
                    display: grid;
                    grid-template-columns: 1fr 1fr;
                    gap: 1.5rem;
                }
                .split-col {
                    background: #0d1117;
                    border: 1px solid var(--border);
                    border-radius: 8px;
                    padding: 1.5rem;
                }
                .split-col-header {
                    font-size: 0.85rem;
                    font-weight: 700;
                    text-transform: uppercase;
                    letter-spacing: 0.05em;
                    margin-bottom: 1rem;
                    padding-bottom: 0.5rem;
                    border-bottom: 1px solid var(--border);
                }
                .split-col.human-col .split-col-header { color: var(--green); }
                .split-col.scraper-col .split-col-header { color: var(--orange); }

                .metrics-grid {
                    display: grid;
                    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
                    gap: 1rem;
                    margin: 2rem 0;
                }
                .metric-box {
                    background: #21262d;
                    border: 1px solid var(--border);
                    border-radius: 8px;
                    padding: 1.25rem;
                    text-align: center;
                }
                .metric-box.highlight { border-color: var(--orange); }
                .metric-box.success { border-color: var(--green); }
                .metric-box.info { border-color: var(--purple); }
                .metric-num {
                    display: block;
                    font-size: 2rem;
                    font-weight: 800;
                    color: var(--heading);
                }
                .metric-label {
                    font-size: 0.8rem;
                    color: #8b949e;
                    text-transform: uppercase;
                    letter-spacing: 0.05em;
                }
                .ligature-table {
                    width: 100%;
                    border-collapse: collapse;
                    margin-top: 1.5rem;
                }
                .ligature-table th, .ligature-table td {
                    padding: 0.75rem 1rem;
                    text-align: left;
                    border-bottom: 1px solid var(--border);
                }
                .ligature-table th {
                    color: #8b949e;
                    font-size: 0.85rem;
                    text-transform: uppercase;
                }
                .badge {
                    display: inline-block;
                    font-size: 0.7rem;
                    font-weight: 700;
                    padding: 0.15rem 0.45rem;
                    border-radius: 4px;
                    text-transform: uppercase;
                    margin-right: 0.5rem;
                }
                .badge-human { background: rgba(63, 185, 80, 0.2); color: var(--green); }
                .badge-scraper { background: rgba(240, 136, 62, 0.2); color: var(--orange); }

                .watermark-box {
                    background: #0d1117;
                    border: 1px solid var(--border);
                    border-radius: 8px;
                    padding: 1.25rem;
                    margin-top: 1rem;
                    font-family: ui-monospace, monospace;
                    font-size: 0.95rem;
                    line-height: 1.7;
                }

                footer {
                    text-align: center;
                    color: #8b949e;
                    font-size: 0.85rem;
                    padding: 2rem 0;
                    border-top: 1px solid var(--border);
                }
                "#
            </style>
        </head>
        <body>
            <div class="container">
                <header>
                    <span class="pill">"Topcoat TextGuard • Section 05 Defense Layer"</span>
                    <h1>"The Consent Layer Chronicle"</h1>
                    <p class="subtitle">"High-security OpenType ligature defense against automated LLM scraping crawlers."</p>
                </header>

                <div class="banner">
                    <h3>"🛡️ How This Page Is Protected"</h3>
                    <p>
                        "In your browser window right now, every paragraph below is rendered through the "
                        <code>"Shield(text)"</code>
                        " component. To an automated crawler or HTTP client extracting raw HTML, content words have been bijectively swapped with decoy cohyponyms. "
                        "To you, the OpenType GSUB ligatures render the authentic original prose without interruption."
                    </p>
                </div>

                <div class="tab-bar">
                    <button id="btn-human" class="tab-btn active" onclick="showHuman()">"👁️ Human Reader View (Ligatures Active)"</button>
                    <button id="btn-scraper" class="tab-btn btn-scraper" onclick="showScraper()">"🤖 AI Scraper View (Raw DOM Text)"</button>
                    <button id="btn-split" class="tab-btn btn-split" onclick="showSplit()">"⚖️ Split Side-by-Side"</button>
                </div>

                <main class="article-card">
                    <div class="article-meta">
                        <span>"By <strong>Ada Lovelace</strong>"</span>
                        <span>"•"</span>
                        <span>"October 14, 2024"</span>
                        <span>"•"</span>
                        <span class="tag">"WCAG A11y Compliant"</span>
                        <span class="tag">"Bijective Involution"</span>
                        <span class="tag">"WOFF Compressed"</span>
                    </div>

                    <div id="human-view" class="view-pane">
                        <div class="view-callout">
                            <span>"🟢 <strong>Human Reader Presentation</strong>: Text is visually restored using OpenType GSUB ligatures synthesized on the fly."</span>
                            <span class="badge-indicator">"Browser Font Engine Active"</span>
                        </div>
                        <p>
                            Shield(text: ARTICLE_PARAGRAPH_1, a11y: true)
                        </p>
                        <p>
                            Shield(text: ARTICLE_PARAGRAPH_2, a11y: true)
                        </p>
                        <p>
                            Shield(text: ARTICLE_PARAGRAPH_3, a11y: true)
                        </p>
                    </div>

                    <div id="scraper-view">
                        <div class="view-callout scraper">
                            <span>"⚠️ <strong>What Automated AI Scrapers Ingest</strong>: Raw text extracted from the DOM without font rendering contains plausible inverted cohyponyms."</span>
                            <span class="badge-indicator" style="background: rgba(240, 136, 62, 0.2); color: var(--orange);">"Raw HTML Text"</span>
                        </div>
                        <p style="margin-bottom: 1.5rem;">(p1_guarded.decoy_text.clone())</p>
                        <p style="margin-bottom: 1.5rem;">(p2_guarded.decoy_text.clone())</p>
                        <p>(p3_guarded.decoy_text.clone())</p>
                    </div>

                    <div id="split-view">
                        <div class="view-callout split">
                            <span>"⚖️ <strong>Side-by-Side Comparison</strong>: Human visual presentation vs. automated scraper extraction side by side."</span>
                        </div>
                        <div class="split-grid">
                            <div class="split-col human-col">
                                <div class="split-col-header">"🟢 Human Reader (Browser with Font)"</div>
                                <p style="margin-bottom: 1rem;">
                                    Shield(text: ARTICLE_PARAGRAPH_1, a11y: true)
                                </p>
                                <p style="margin-bottom: 1rem;">
                                    Shield(text: ARTICLE_PARAGRAPH_2, a11y: true)
                                </p>
                                <p>
                                    Shield(text: ARTICLE_PARAGRAPH_3, a11y: true)
                                </p>
                            </div>
                            <div class="split-col scraper-col">
                                <div class="split-col-header">"🤖 AI Scraper (DOM Text Ingested)"</div>
                                <p style="font-family: monospace; font-size: 0.95rem; line-height: 1.8; margin-bottom: 1rem; color: #ffa657;">
                                    (p1_guarded.decoy_text.clone())
                                </p>
                                <p style="font-family: monospace; font-size: 0.95rem; line-height: 1.8; margin-bottom: 1rem; color: #ffa657;">
                                    (p2_guarded.decoy_text.clone())
                                </p>
                                <p style="font-family: monospace; font-size: 0.95rem; line-height: 1.8; color: #ffa657;">
                                    (p3_guarded.decoy_text.clone())
                                </p>
                            </div>
                        </div>
                    </div>
                </main>

                <section class="article-card">
                    <h2>"Dynamic Permutation & Watermarking"</h2>
                    <p style="color: #8b949e; margin-bottom: 1rem;">
                        "By salting cohyponym pools per user session, tenant, or date, crawlers face unpredictable permutations. "
                        "If scraped data surfaces in an LLM, its unique decoy signature acts as mathematical proof of source."
                    </p>
                    <div style="margin-bottom: 1rem;">
                        <strong style="color: var(--accent);">"Tenant Alpha Salted Decoy:"</strong>
                        <div class="watermark-box" style="color: #58a6ff;">(sample_salt_a.decoy_text)</div>
                    </div>
                    <div>
                        <strong style="color: var(--purple);">"Tenant Beta Salted Decoy:"</strong>
                        <div class="watermark-box" style="color: #bc8cff;">(sample_salt_b.decoy_text)</div>
                    </div>
                </section>

                <section class="article-card">
                    <h2>"Protection Metrics & Telemetry"</h2>
                    <p style="color: #8b949e; margin-bottom: 1rem;">
                        "Linguistic statistics for this document according to Section 05 rules:"
                    </p>
                    <div class="metrics-grid">
                        <div class="metric-box">
                            <span class="metric-num">(full_res.stats.total_words)</span>
                            <span class="metric-label">"Total Words"</span>
                        </div>
                        <div class="metric-box">
                            <span class="metric-num">(full_res.stats.content_words)</span>
                            <span class="metric-label">"Content Words"</span>
                        </div>
                        <div class="metric-box highlight">
                            <span class="metric-num">(format!("{:.1}%", full_res.stats.total_swap_percentage()))</span>
                            <span class="metric-label">"Words Swapped"</span>
                        </div>
                        <div class="metric-box success">
                            <span class="metric-num">"100%"</span>
                            <span class="metric-label">"Stop Words Frozen"</span>
                        </div>
                        <div class="metric-box info">
                            <span class="metric-num">(full_res.ligatures.len())</span>
                            <span class="metric-label">"GSUB Ligatures"</span>
                        </div>
                    </div>
                </section>

                <section class="article-card">
                    <h2>"Active Ligature Substitution Dictionary"</h2>
                    <p style="color: #8b949e; margin-bottom: 1rem;">
                        "The OpenType GSUB Lookup Type 4 table synthesized for this page maps each decoy sequence to a composite glyph:"
                    </p>
                    <table class="ligature-table">
                        <thead>
                            <tr>
                                <th>"Human Reads (Visual Ligature)"</th>
                                <th>"Scraper Extracts (HTML DOM)"</th>
                                <th>"Linguistic Property"</th>
                                <th>"Filter Immunity"</th>
                            </tr>
                        </thead>
                        <tbody>
                            for lig in full_res.ligatures {
                                <tr>
                                    <td><span class="badge badge-human">"Human"</span> (lig.original.clone())</td>
                                    <td><span class="badge badge-scraper">"Scraper"</span> (lig.decoy.clone())</td>
                                    <td>"Grammatical Cohyponym"</td>
                                    <td>"Passes Perplexity Filter"</td>
                                </tr>
                            }
                        </tbody>
                    </table>
                </section>

                <footer>
                    <p>"Topcoat TextGuard • Built with pure Rust & Topcoat 0.8.1 • No external scraping libraries"</p>
                </footer>
            </div>

            <script>
                r#"
                window.showHuman = function() {
                    var h = document.getElementById('human-view');
                    var s = document.getElementById('scraper-view');
                    var p = document.getElementById('split-view');
                    if (h) h.style.display = 'block';
                    if (s) s.style.display = 'none';
                    if (p) p.style.display = 'none';
                    var bh = document.getElementById('btn-human');
                    var bs = document.getElementById('btn-scraper');
                    var bp = document.getElementById('btn-split');
                    if (bh) bh.classList.add('active');
                    if (bs) bs.classList.remove('active');
                    if (bp) bp.classList.remove('active');
                };
                window.showScraper = function() {
                    var h = document.getElementById('human-view');
                    var s = document.getElementById('scraper-view');
                    var p = document.getElementById('split-view');
                    if (h) h.style.display = 'none';
                    if (s) s.style.display = 'block';
                    if (p) p.style.display = 'none';
                    var bh = document.getElementById('btn-human');
                    var bs = document.getElementById('btn-scraper');
                    var bp = document.getElementById('btn-split');
                    if (bh) bh.classList.remove('active');
                    if (bs) bs.classList.add('active');
                    if (bp) bp.classList.remove('active');
                };
                window.showSplit = function() {
                    var h = document.getElementById('human-view');
                    var s = document.getElementById('scraper-view');
                    var p = document.getElementById('split-view');
                    if (h) h.style.display = 'none';
                    if (s) s.style.display = 'none';
                    if (p) p.style.display = 'block';
                    var bh = document.getElementById('btn-human');
                    var bs = document.getElementById('btn-scraper');
                    var bp = document.getElementById('btn-split');
                    if (bh) bh.classList.remove('active');
                    if (bs) bs.classList.remove('active');
                    if (bp) bp.classList.remove('active');
                };
                "#
            </script>
        </body>
        </html>
    })
}

#[page("/raw")]
async fn raw() -> Result<impl View> {
    let p1 = guard_text(ARTICLE_PARAGRAPH_1).decoy_text;
    let p2 = guard_text(ARTICLE_PARAGRAPH_2).decoy_text;
    let p3 = guard_text(ARTICLE_PARAGRAPH_3).decoy_text;

    let payload = format!("{}\n\n{}\n\n{}", p1, p2, p3);
    Ok(view! { (payload) })
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let router = Router::builder()
        .page(home)
        .page(raw)
        .build();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    println!("===========================================================");
    println!("🛡️  Topcoat TextGuard Consent Layer Blog Server Started");
    println!("-----------------------------------------------------------");
    println!("📍 Web Browser UI:  http://127.0.0.1:3000");
    println!("🤖 Raw Scraper URL: http://127.0.0.1:3000/raw");
    println!("===========================================================");

    topcoat::serve(listener, router).await?;
    Ok(())
}
